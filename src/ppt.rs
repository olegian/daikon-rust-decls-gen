use crate::{
    decls::{FIELD_LENGTH, RETURN_VAR_NAME},
    fields::{DecType, ParentRelationType, ProgramPointType, VarKind, VariableDecl},
    globals::{ConstSource, Global},
    vars::VarName,
};

#[derive(Debug)]
pub struct ProgramPoint {
    pub ppt_type: ProgramPointType,
    pub variables: std::collections::BTreeMap<String, VariableDecl>,
    pub parents: std::collections::BTreeMap<String, (ParentRelationType, u64)>,

    // Only useful during construction from source,
    // to add globals only to ppts that can access them.
    local_def_id: Option<rustc_hir::def_id::LocalDefId>,
}

impl ProgramPoint {
    /// Construct a new, empty program point of type ENTER
    pub fn enter(name: &str, ldid: rustc_hir::def_id::LocalDefId) -> (String, Self) {
        (
            format!("{name}:::ENTER"),
            Self {
                ppt_type: ProgramPointType::Enter,
                variables: std::collections::BTreeMap::new(),
                parents: std::collections::BTreeMap::new(),
                local_def_id: Some(ldid),
            },
        )
    }

    /// Construct a new, empty program point of type SUBEXIT
    pub fn subexit(name: &str, ldid: rustc_hir::def_id::LocalDefId, id: u64) -> (String, Self) {
        (
            format!("{name}:::EXIT{id}"),
            Self {
                ppt_type: ProgramPointType::ExitNN(id),
                variables: std::collections::BTreeMap::new(),
                parents: std::collections::BTreeMap::new(),
                local_def_id: Some(ldid),
            },
        )
    }

    /// Construct a new, empty program point of type EXIT
    pub fn exit(name: &str, ldid: rustc_hir::def_id::LocalDefId) -> (String, Self) {
        (
            format!("{name}:::EXIT"),
            Self {
                ppt_type: ProgramPointType::Exit,
                variables: std::collections::BTreeMap::new(),
                parents: std::collections::BTreeMap::new(),
                local_def_id: Some(ldid),
            },
        )
    }

    /// Simple constructor
    pub fn new(
        ppt_type: ProgramPointType,
        variables: std::collections::BTreeMap<String, VariableDecl>,
        parents: std::collections::BTreeMap<String, (ParentRelationType, u64)>,
    ) -> Self {
        Self {
            ppt_type,
            variables,
            parents,
            local_def_id: None,
        }
    }

    pub fn get_var(&self, qualified_var_path: &str) -> Option<&VariableDecl> {
        self.variables.get(qualified_var_path)
    }

    pub fn get_var_mut(&mut self, qualified_var_path: &str) -> Option<&mut VariableDecl> {
        self.variables.get_mut(qualified_var_path)
    }

    /// Register the program point with name `parent_ppt` as a parent of
    /// this program point, with the given relation type and id.
    pub fn with_parent(
        mut self,
        parent_ppt: String,
        relation_type: ParentRelationType,
        relation_id: &mut u64,
    ) -> Self {
        if self.parents.contains_key(&parent_ppt) {
            panic!("{parent_ppt} is already a parent!")
        }

        self.parents
            .insert(parent_ppt, (relation_type, *relation_id));
        *relation_id += 1;

        self
    }

    /// Add the return value (and all recursive values inside the return)
    /// to this program point. If the return type is unit, does nothing.
    pub fn with_fn_return<'b>(
        mut self,
        tcx: rustc_middle::ty::TyCtxt<'b>,
        ret_ty: rustc_middle::ty::Ty<'b>,
        max_recursive_depth: Option<usize>,
    ) -> Self {
        if ret_ty.is_unit() {
            return self;
        }
        self.add_var(
            tcx,
            VarName::new(RETURN_VAR_NAME),
            &ret_ty,
            None,
            VarKind::Return,
            max_recursive_depth,
            false,
            None,
            false,
        );

        self
    }

    /// Add all function inputs (formals) to this program point.
    ///
    /// `inputs` yields items (var_name, var_type, is_uninit). When `is_uninit`
    /// is true, the formal (and every recursively-expanded sub-variable) is
    /// tagged with `constant UNINITIALIZED`, signaling that the value has
    /// been dropped/moved-out by this program point.
    pub fn with_fn_inputs<'a, 'b: 'a>(
        mut self,
        tcx: rustc_middle::ty::TyCtxt<'b>,
        // i really think there is a better way to represent this, but
        // because we are pulling names off the HIR body and types off the MIR,
        // i'm not sure if there is a unified existing representation for it.
        inputs: impl Iterator<Item = (String, &'a rustc_middle::ty::Ty<'b>, bool)>,
        max_recursive_depth: Option<usize>,
    ) -> Self {
        for (name, ty, is_uninit) in inputs {
            self.add_var(
                tcx,
                VarName::new(name),
                ty,
                None,
                VarKind::Variable,
                max_recursive_depth,
                false,
                None,
                is_uninit,
            );
        }

        self
    }

    /// Add var-decls for every entry in `globals` that is visible to this
    /// program point. Evaluates constants and non-mut static globals. Note,
    /// this function must run after all subexit points have been constructed,
    /// as to not query a stolen MIR.
    pub fn add_globals<'tcx>(
        &mut self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        evs: &rustc_middle::middle::privacy::EffectiveVisibilities,
        globals: &[Global],
        max_recursive_depth: Option<usize>,
    ) {
        let Some(ppt_def_id) = self.local_def_id else {
            panic!(
                "Cannot add globals to program points without knowledge of constant visibility. \
                 Adding globals requires building a decls file from source .rs files."
            )
        };

        for global in globals {
            let global_vis = evs
                .effective_vis(global.ldid)
                .map(|ev| {
                    ev.at_level(rustc_middle::middle::privacy::Level::Reexported)
                        .to_def_id()
                })
                .unwrap_or_else(|| {
                    // evs will only contain items which are visible from outside of
                    // their own parent modules. Not all items are therefore stored in
                    // evs, as some of them are private / otherwise do not escape the parent.

                    // these private consts should therefore have Restricted visibiilty,
                    // which represents that they can only be seen in the module they are
                    // defined in.
                    rustc_middle::ty::Visibility::Restricted(
                        tcx.parent_module_from_def_id(global.ldid).to_def_id(),
                    )
                });

            // actually check if this global is visible to this ppt
            if global_vis.is_accessible_from(ppt_def_id, tcx) {
                // ... and success! we have a visible global!
                // type check it, and start recursive descent,
                // evaluating it as we go, if possible, as determined
                // by the const_source.

                let ty = tcx
                    .type_of(global.ldid)
                    .instantiate_identity()
                    .skip_normalization();
                let const_source = global.const_source(tcx);

                self.add_var(
                    tcx,
                    VarName::new(global.name.clone()),
                    &ty,
                    None,
                    VarKind::Variable,
                    max_recursive_depth,
                    false,
                    const_source,
                    false,
                )
            }
        }
    }

    /// Emit a record for the variable name: ty. If the type is a
    /// compound type, recursively emit records for its contained variables.
    ///
    /// `name` is the fully-qualified variable name (e.g. "arr[..].field").
    /// `parent` is the enclosing variable's fully-qualified name (e.g. "arr[..]"),
    ///    or None (in which case the name` var is the root.)
    /// `var_kind` describes how this variable relates to its parent, described by the
    ///    .decls file format specification.
    /// `in_array` is sticky and stops reporting sequences of sequences.
    ///    It should always be initially set to false.
    /// `const_source`, when present, evaluates the compile-time constant described
    ///    the variable. each leaf is tagged with its constant value.
    ///    further, when Some, arrays/slices are expanded into per-element
    ///    var-decls (p[0], p[1], ...) instead of the collapsed [..] sequence.
    /// `is_uninit`, when true, tags every emitted var-decl (this one and every
    ///    recursively expanded sub-variable) with `constant UNINITIALIZED`. Used
    ///    for formals that have been dropped/moved-out by this program point.
    fn add_var<'b>(
        &mut self,
        tcx: rustc_middle::ty::TyCtxt<'b>,
        name: VarName,
        ty: &rustc_middle::ty::Ty<'b>,
        parent: Option<VarName>,
        var_kind: VarKind,
        remaining_recursive_depth: Option<usize>,
        in_array: bool,
        const_source: Option<ConstSource<'b>>,
        is_uninit: bool,
    ) {
        // stop expanding vars if we have hit the max depth.
        if let Some(remaining_depth) = remaining_recursive_depth
            && remaining_depth == 0
        {
            return;
        }

        // Peel all references. Each peel that crosses a Scalar(Ptr) const
        // source dereferences the pointer to land on the pointee bytes;
        // Slice/Indirect sources already describe the referent directly.
        // Refs to unsized types (&[T], &str) whose source is Indirect
        // are stored as the encoded fat pointer (data ptr + length); read it
        // out and turn it into a Slice source pointing at the elements.
        let mut ty = ty;
        let mut const_source = const_source;
        while let rustc_type_ir::TyKind::Ref(_, inner, _) = ty.kind() {
            // properly shift offsets in const source, such that after
            // all indirection is removed, const_source refers to the appropriate allocation
            if let Some(src) = const_source {
                const_source = match (src, inner.kind()) {
                    // the const_source is a simple thin pointer...
                    (ConstSource::Scalar(rustc_middle::mir::interpret::Scalar::Ptr(_, _)), _) => {
                        // ...in stripping the reference, we just had to deref the pointer.
                        src.deref_ptr(tcx)
                    }

                    // the const_source is a more complicated, fat pointer...
                    (
                        ConstSource::Indirect(_, _),
                        rustc_type_ir::TyKind::Slice(_) | rustc_type_ir::TyKind::Str,
                    ) => {
                        // ... in which case we have to create a type we can actually
                        // read and interpret values from
                        src.load_fat_ptr_to_slice(tcx)
                    }
                    _ => Some(src),
                };
            }

            // peel refs on ty
            ty = inner;
        }

        // Any variable inside (or equal to) an array sequence has multiple
        // values, so it gets the <array 1> tag set, alongside any contained value.
        let in_array = in_array || matches!(var_kind, VarKind::Array);
        let dec_type = DecType::from_ty(&tcx, ty);
        let mut var_decl =
            VariableDecl::new(var_kind, DecType::from_ty(&tcx, ty)).with_enclosing_var(parent);
        if in_array {
            var_decl = var_decl.with_array(Some(1));
        }

        // Uninit formals get a UNINITIALIZED tag on every emitted var-decl,
        // overriding any const-source tagging (in practice const_source is
        // always None for uninit formals anyway).
        if is_uninit {
            var_decl = var_decl.with_constant(Some("UNINITIALIZED".to_string()));
        }

        // If a const source is available, attach a constant tag for this decl.
        // Primitives get their actual value
        // &str gets a decoded string;
        // FIXME: compound types get a TEMP_PTR placeholder for now, but are then
        // recursively expanded
        if !is_uninit && let Some(src) = &const_source {
            let constant = match ty.kind() {
                rustc_type_ir::TyKind::Bool
                | rustc_type_ir::TyKind::Char
                | rustc_type_ir::TyKind::Int(_)
                | rustc_type_ir::TyKind::Uint(_)
                | rustc_type_ir::TyKind::Float(_) => src.read_leaf(tcx, ty),
                rustc_type_ir::TyKind::Str => src.read_str(),
                // String::new() is the only const fn String constructor,
                // so any const-evaluatable String is the empty string.
                // in the future, if that ever changes, this will need to as well.
                _ if matches!(dec_type, DecType::String) => Some("\"\"".to_string()),
                _ if matches!(dec_type, DecType::Compound(_)) => {
                    Some("TEMPORARY_PTR_PLACEHOLDER".to_string())
                }
                _ => None,
            };
            if constant.is_some() {
                var_decl = var_decl.with_constant(constant);
            }
        }

        self.variables.insert(name.as_str().to_string(), var_decl);

        match ty.kind() {
            // impossible, we peeled all refs already.
            rustc_type_ir::TyKind::Ref(_, _, _) => unreachable!(),

            // Leaf types, the record we just inserted is all there is to say.
            rustc_type_ir::TyKind::Bool
            | rustc_type_ir::TyKind::Char
            | rustc_type_ir::TyKind::Int(_)
            | rustc_type_ir::TyKind::Uint(_)
            | rustc_type_ir::TyKind::Float(_)
            | rustc_type_ir::TyKind::Str => {}

            // Unimplemented types, could be worth considering later...
            // for now let them fall through?
            rustc_type_ir::TyKind::Foreign(_)
            | rustc_type_ir::TyKind::Pat(_, _)
            | rustc_type_ir::TyKind::RawPtr(_, _)
            | rustc_type_ir::TyKind::FnDef(_, _)
            | rustc_type_ir::TyKind::FnPtr(_, _)
            | rustc_type_ir::TyKind::UnsafeBinder(_)
            | rustc_type_ir::TyKind::Dynamic(_, _)
            | rustc_type_ir::TyKind::Closure(_, _)
            | rustc_type_ir::TyKind::CoroutineClosure(_, _)
            | rustc_type_ir::TyKind::Coroutine(_, _)
            | rustc_type_ir::TyKind::CoroutineWitness(_, _)
            | rustc_type_ir::TyKind::Never
            | rustc_type_ir::TyKind::Alias(_)
            | rustc_type_ir::TyKind::Param(_)
            | rustc_type_ir::TyKind::Bound(_, _)
            | rustc_type_ir::TyKind::Placeholder(_)
            | rustc_type_ir::TyKind::Infer(_)
            | rustc_type_ir::TyKind::Error(_) => {}

            // Arrays. With a const source, expand into per-element decls
            // name[0]..name[N-1]. Without a source, treat this as a var and
            // emit the [..] sequence.
            rustc_type_ir::TyKind::Array(inner, count_const) => {
                let static_count = count_const.try_to_target_usize(tcx);
                let len_name = name.project_field(FIELD_LENGTH);
                let mut len_decl =
                    VariableDecl::new(VarKind::Field(FIELD_LENGTH.to_string()), DecType::Usize)
                        .with_enclosing_var(Some(name.clone()));
                if is_uninit {
                    len_decl.set_constant(Some("UNINITIALIZED".to_string()));
                }

                if let (Some(src), Some(n)) = (const_source, static_count) {
                    len_decl.set_constant(Some(n.to_string()));

                    for i in 0..n {
                        let child_src = src.project_field(tcx, ty, i as usize);
                        let rel = format!("[{}]", i);
                        let child_name = name.project_index(i as usize);
                        self.add_var(
                            tcx,
                            child_name,
                            inner,
                            Some(name.clone()),
                            VarKind::Field(rel),
                            remaining_recursive_depth.map(|r| r - 1),
                            in_array,
                            child_src,
                            is_uninit,
                        );
                    }
                } else {
                    if in_array {
                        return;
                    }

                    let elem_name = name.project_slice();
                    self.add_var(
                        tcx,
                        elem_name,
                        inner,
                        Some(name),
                        VarKind::Array,
                        remaining_recursive_depth.map(|r| r - 1),
                        in_array,
                        None,
                        is_uninit,
                    );
                }

                self.variables.insert(len_name.into_string(), len_decl);
            }

            // Slices. Handled very similarly to arrays, see above
            rustc_type_ir::TyKind::Slice(inner) => {
                let slice_len = const_source.and_then(|s| s.slice_len());
                let len_name = name.project_field(FIELD_LENGTH);
                let mut len_decl =
                    VariableDecl::new(VarKind::Field(FIELD_LENGTH.to_string()), DecType::Usize)
                        .with_enclosing_var(Some(name.clone()));
                if is_uninit {
                    len_decl.set_constant(Some("UNINITIALIZED".to_string()));
                }

                if let (Some(src), Some(n)) = (const_source, slice_len) {
                    len_decl.set_constant(Some(n.to_string()));

                    for i in 0..n {
                        let child_src = src.project_slice_elem(tcx, *inner, i);
                        let rel = format!("[{}]", i);
                        let child_name = name.project_index(i as usize);
                        self.add_var(
                            tcx,
                            child_name,
                            inner,
                            Some(name.clone()),
                            VarKind::Field(rel),
                            remaining_recursive_depth.map(|r| r - 1),
                            in_array,
                            child_src,
                            is_uninit,
                        );
                    }
                } else {
                    if in_array {
                        return;
                    }

                    let elem_name = name.project_slice();
                    self.add_var(
                        tcx,
                        elem_name,
                        inner,
                        Some(name),
                        VarKind::Array,
                        remaining_recursive_depth.map(|r| r - 1),
                        in_array,
                        None,
                        is_uninit,
                    );
                }

                self.variables.insert(len_name.into_string(), len_decl);
            }

            // Tuples emit a var decl per field.
            rustc_type_ir::TyKind::Tuple(inner_tys) => {
                for (i, inner) in inner_tys.iter().enumerate() {
                    let rel = i.to_string();
                    let child_name = name.project_field(&rel);
                    let child_src = const_source.and_then(|s| s.project_field(tcx, ty, i));
                    self.add_var(
                        tcx,
                        child_name,
                        &inner,
                        Some(name.clone()),
                        VarKind::Field(rel),
                        remaining_recursive_depth.map(|remaining| remaining - 1),
                        in_array,
                        child_src,
                        is_uninit,
                    );
                }
            }

            // Compound types
            rustc_type_ir::TyKind::Adt(adt_def, generics) => {
                let adt_did = adt_def.did();
                if !adt_did.is_local() {
                    // Some external types are special-cased, namely:
                    // Vec, Box, and Range*. Handle those, otherwise skip.
                    // i don't think any of these types can be evaluated
                    // in a const context... leaving it as None for now.
                    self.maybe_foreign_special_case_ty(
                        tcx,
                        name,
                        adt_did,
                        &generics[..],
                        remaining_recursive_depth,
                        in_array,
                        is_uninit,
                    );
                    return;
                }

                match adt_def.adt_kind() {
                    // the compound type is a struct!
                    rustc_middle::ty::AdtKind::Struct => {
                        for (i, field) in adt_def.all_fields().enumerate() {
                            let field_name = field.ident(tcx).name.to_string();
                            let field_ty = tcx
                                .type_of(field.did)
                                .instantiate(tcx, generics)
                                .skip_normalization();

                            let child_name = name.project_field(&field_name);
                            let child_src = const_source.and_then(|s| s.project_field(tcx, ty, i));
                            self.add_var(
                                tcx,
                                child_name,
                                &field_ty,
                                Some(name.clone()),
                                VarKind::Field(field_name),
                                remaining_recursive_depth.map(|remaining| remaining - 1),
                                in_array,
                                child_src,
                                is_uninit,
                            );
                        }
                    }

                    // the compound type is an enum!
                    rustc_middle::ty::AdtKind::Enum => {
                        // Flatten all enum variants by prefixing each variant's fields
                        // with ::Variant. Const decomposition for enums (which depends
                        // on the discriminant, as in only one variant would make up the
                        // constant) isn't implemented yet.
                        // reading discriminants is just kinda hard... rust chooses to encode
                        // the discriminant in some fancy ways (lookup Variants::Multiple
                        // { tag_encoding: TagEncoding})), the tag encoding type is
                        // necessary to know how to project into the union...
                        for variant in adt_def.variants() {
                            let variant_name = variant.ident(tcx).name.to_string();
                            for field in &variant.fields {
                                let field_name = field.ident(tcx).name.to_string();
                                let field_ty = tcx
                                    .type_of(field.did)
                                    .instantiate(tcx, generics)
                                    .skip_normalization();

                                let rel = format!("{}.{}", variant_name, field_name);
                                let child_name = name
                                    .project_variant(&variant_name)
                                    .project_field(&field_name);
                                self.add_var(
                                    tcx,
                                    child_name,
                                    &field_ty,
                                    Some(name.clone()),
                                    VarKind::Field(rel),
                                    remaining_recursive_depth.map(|remaining| remaining - 1),
                                    in_array,
                                    None,
                                    is_uninit,
                                );
                            }
                        }
                    }
                    rustc_middle::ty::AdtKind::Union => {
                        panic!("Union parameter types are not supported.")
                    }
                }
            }
        }
    }

    fn maybe_foreign_special_case_ty<'tcx>(
        &mut self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        name: VarName,
        adt_did: rustc_span::def_id::DefId,
        adt_generics: &[rustc_middle::ty::GenericArg<'tcx>],
        remaining_recursive_depth: Option<usize>,
        in_array: bool,
        is_uninit: bool,
    ) {
        let lang_items = tcx.lang_items();
        if tcx.is_diagnostic_item(rustc_span::symbol::sym::Vec, adt_did) {
            // Vec type, treat similar to array. if P: Vec<u32> include var decls for:
            // p  <-- already included by add_var
            // p.length  <-- varkind field
            // p[..]  <-- varkind array

            if in_array {
                // see array handling in self.add_var
                return;
            }

            let len_name = name.project_field(FIELD_LENGTH);
            let mut var_decl =
                VariableDecl::new(VarKind::Field(FIELD_LENGTH.to_string()), DecType::Usize)
                    .with_enclosing_var(Some(name.clone()));
            if in_array {
                var_decl = var_decl.with_array(Some(1));
            }
            if is_uninit {
                var_decl = var_decl.with_constant(Some("UNINITIALIZED".to_string()));
            }
            self.variables.insert(len_name.into_string(), var_decl);

            let elem_ty = adt_generics[0]
                .as_type()
                .expect("Found Vec<_> with no specified element type");
            let elem_name = name.project_slice();
            self.add_var(
                tcx,
                elem_name,
                &elem_ty,
                Some(name),
                VarKind::Array,
                remaining_recursive_depth.map(|remaining| remaining - 1),
                in_array,
                None,
                is_uninit,
            );
        } else if adt_did
            == lang_items
                .owned_box()
                .expect("Unable to find def id of std box.")
        {
            // Box type, if p: Box<T>, include var decls for
            // p: hashcode. <-- record included by add_var
            // *p: T (inline with the enclosing var, in other words
            //        a.b.c.*p could be the full path)
            // FIXME: should it instead be something like *(a.b.c.p)?

            // Box guaranteed to have exactly one generic type, the pointee type
            let pointee_ty = adt_generics[0]
                .as_type()
                .expect("Found Box<_> with no specified pointee type");
            let pointee = name.project_deref();

            // FIXME: I don't think the varkind here should be Variable. Not sure what else
            // to do though, maybe keep it whatever it was previously?
            self.add_var(
                tcx,
                pointee,
                &pointee_ty,
                Some(name),
                VarKind::Variable,
                remaining_recursive_depth.map(|remain| remain - 1),
                in_array,
                None,
                is_uninit,
            );
        } else if adt_did
            == lang_items
                .range_full_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range Full (..)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_from_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range From (a..)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_from_copy_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range From + Copy (a.. such that a: Copy)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_to_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range To (..b)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_to_inclusive_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range To Inclusive (..=b)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_to_inclusive_copy_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range To Inclusive (..=b such that b: Copy)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range (a..b)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_copy_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range Copy (a..b such that a, b: Copy)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_inclusive_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range Inclusive (a..=b)
            eprintln!("Ranges not yet special-cased, skipping...")
        } else if adt_did
            == lang_items
                .range_inclusive_copy_struct()
                .expect("Unable to find def id of RangeFull")
        {
            // Range Inclusive + Copy (a..=b such that a, b: Copy)
            eprintln!("Ranges not yet special-cased, skipping...")
        }
    }
}
