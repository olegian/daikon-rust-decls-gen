use crate::fields::{DecType, ParentRelationType, ProgramPointType, VarKind, VariableDecl};

#[derive(Debug)]
pub struct ProgramPoint {
    pub ppt_type: ProgramPointType,
    pub variables: std::collections::HashMap<String, VariableDecl>,
    pub parents: std::collections::HashMap<String, (ParentRelationType, u64)>,
}

impl ProgramPoint {
    /// Construct a new, empty program point of type ENTER
    pub fn enter(name: &str) -> (String, Self) {
        (
            format!("{name}:::ENTER"),
            Self {
                ppt_type: ProgramPointType::Enter,
                variables: std::collections::HashMap::new(),
                parents: std::collections::HashMap::new(),
            },
        )
    }

    /// Construct a new, empty program point of type SUBEXIT
    pub fn subexit(name: &str, id: u64) -> (String, Self) {
        (
            format!("{name}:::EXIT{id}"),
            Self {
                ppt_type: ProgramPointType::ExitNN(id),
                variables: std::collections::HashMap::new(),
                parents: std::collections::HashMap::new(),
            },
        )
    }

    /// Construct a new, empty program point of type EXIT
    pub fn exit(name: &str) -> (String, Self) {
        (
            format!("{name}:::EXIT"),
            Self {
                ppt_type: ProgramPointType::Exit,
                variables: std::collections::HashMap::new(),
                parents: std::collections::HashMap::new(),
            },
        )
    }

    pub fn get_var(&self, qualified_var_path: &str) -> Option<&VariableDecl> {
        self.variables.get(qualified_var_path)
    }

    pub fn get_var_mut(&mut self, qualified_var_path: &str) -> Option<&mut VariableDecl> {
        self.variables.get_mut(qualified_var_path)
    }

    /// Register the program point with name `parent_ppt` as a parent of
    /// this program point, with the given relation type and id.
    pub fn add_parent(
        &mut self,
        parent_ppt: String,
        relation_type: ParentRelationType,
        relation_id: u64,
    ) {
        if self.parents.contains_key(&parent_ppt) {
            panic!("{parent_ppt} is already a parent!")
        }

        self.parents
            .insert(parent_ppt, (relation_type, relation_id));
    }

    /// Add the return value (and all recursive values inside the return)
    /// to this program point. If the return type is unit, does nothing.
    pub fn include_fn_return<'b>(
        &mut self,
        tcx: &rustc_middle::ty::TyCtxt<'b>,
        ret_ty: rustc_middle::ty::Ty<'b>,
        max_recursive_depth: Option<usize>,
    ) {
        if ret_ty.is_unit() {
            return;
        }
        self.add_var(
            tcx,
            "return".to_string(),
            ret_ty,
            None,
            VarKind::Return,
            max_recursive_depth,
            false,
        );
    }

    /// Add all function inputs (formals) to this program point.
    ///
    /// `inputs` is an iterator that nets items (var_name: String, var_type: mir::Ty).
    pub fn include_fn_inputs<'a, 'b>(
        &mut self,
        tcx: &'a rustc_middle::ty::TyCtxt<'b>,
        // i really think there is a better way to represent this, but
        // because we are pulling names off the HIR body and types off the MIR,
        // i'm not sure if there is a unified existing representation for it.
        inputs: impl Iterator<Item = (String, &'a rustc_middle::ty::Ty<'b>)>,
        max_recursive_depth: Option<usize>,
    ) {
        for (name, ty) in inputs {
            self.add_var(
                tcx,
                name,
                *ty,
                None,
                VarKind::Variable,
                max_recursive_depth,
                false,
            );
        }
    }

    /// Emit a record for the variable `name: ty`. If the type is a
    /// compound type, recursively emit records for its contained variables.
    ///
    /// `name` is the fully-qualified variable name (e.g. "arr[..].field").
    /// `parent` is the enclosing variable's fully-qualified name, or `None`.
    /// `var_kind` describes how this variable relates to its parent
    /// `in_array` used to stop reporting sequences of sequences, sticky parameter.
    fn add_var<'b>(
        &mut self,
        tcx: &rustc_middle::ty::TyCtxt<'b>,
        name: String,
        ty: rustc_middle::ty::Ty<'b>,
        parent: Option<String>,
        var_kind: VarKind,
        remaining_recursive_depth: Option<usize>,
        in_array: bool,
    ) {
        if let Some(remaining_depth) = remaining_recursive_depth
            && remaining_depth == 0
        {
            return;
        }

        // Peel all references to the current type
        let mut ty = ty;
        while let rustc_type_ir::TyKind::Ref(_, inner, _) = ty.kind() {
            ty = *inner;
        }

        // Any variable inside (or equal to) an array sequence has multiple
        // values, so it gets the <array 1> tag set, alongside any contained value.
        let in_array = in_array || matches!(var_kind, VarKind::Array);
        self.variables.insert(
            name.clone(),
            VariableDecl::new(
                var_kind,
                DecType::from_ty(ty),
                parent,
                if in_array { 1 } else { 0 },
                None,
            ),
        );

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

            // Arrays and slices emit `.length` (a field of the array pointer)
            // and then recurse into the actual sequence.
            rustc_type_ir::TyKind::Array(inner, _) | rustc_type_ir::TyKind::Slice(inner) => {
                if in_array {
                    // if we are already in an array, do not recurse deeper.
                    // consider p: [Struct] where Struct.x --> [u32; N]
                    // p
                    // p.length
                    // p[..]   <- always has a flat repr
                    // p[..].x  <-- could have a flat repr, as long as it's not an array itself
                    // p[..].x[..]  <-- cannot have a flat repr.
                    // for now, because flattening out the sequence of
                    // sequences for daikon is impossible, we won't report the vars
                    // in the second sequence. If higher dim-arrays are ever supported,
                    // just remove this clause and everything should work fine.
                    return;
                }

                let len_name = format!("{}.length", name);
                self.variables.insert(
                    len_name,
                    VariableDecl::new(
                        VarKind::Field("length".to_string()),
                        DecType::Usize,
                        Some(name.clone()),
                        if in_array { 1 } else { 0 },
                        None,
                    ),
                );

                let elem_name = format!("{}[..]", name);
                self.add_var(
                    tcx,
                    elem_name,
                    *inner,
                    Some(name),
                    VarKind::Array,
                    remaining_recursive_depth.map(|remaining| remaining - 1),
                    in_array,
                );
            }

            // Tuples emit each a var decl for all fields
            rustc_type_ir::TyKind::Tuple(inner_tys) => {
                for (i, inner) in inner_tys.iter().enumerate() {
                    let rel = i.to_string();
                    let child_name = format!("{}.{}", name, rel);
                    self.add_var(
                        tcx,
                        child_name,
                        inner,
                        Some(name.clone()),
                        VarKind::Field(rel),
                        remaining_recursive_depth.map(|remaining| remaining - 1),
                        in_array,
                    );
                }
            }

            // Compound types
            rustc_type_ir::TyKind::Adt(adt_def, generics) => {
                let adt_did = adt_def.did();
                if !adt_did.is_local() {
                    // Some external types are special-cased, namely:
                    // Vec, Box, and Range*. Handle those, otherwise skip.
                    self.maybe_foreign_special_case_ty(
                        tcx,
                        name,
                        adt_did,
                        &generics[..],
                        // parent,
                        // var_kind,
                        remaining_recursive_depth,
                        in_array,
                    );
                    return;
                }

                match adt_def.adt_kind() {
                    rustc_middle::ty::AdtKind::Struct => {
                        for field in adt_def.all_fields() {
                            let field_name = field.ident(*tcx).name.to_string();
                            let field_ty = tcx
                                .type_of(field.did)
                                .instantiate(*tcx, generics)
                                .skip_normalization();

                            let child_name = format!("{}.{}", name, field_name);
                            self.add_var(
                                tcx,
                                child_name,
                                field_ty,
                                Some(name.clone()),
                                VarKind::Field(field_name),
                                remaining_recursive_depth.map(|remaining| remaining - 1),
                                in_array,
                            );
                        }
                    }
                    rustc_middle::ty::AdtKind::Enum => {
                        // Flatten all enum variants by prefixing each variant's fields
                        // with ::Variant?
                        for variant in adt_def.variants() {
                            let variant_name = variant.ident(*tcx).name.to_string();
                            for field in &variant.fields {
                                let field_name = field.ident(*tcx).name.to_string();
                                let field_ty = tcx
                                    .type_of(field.did)
                                    .instantiate(*tcx, generics)
                                    .skip_normalization();

                                let rel = format!("{}.{}", variant_name, field_name);
                                let child_name =
                                    format!("{}::{}.{}", name, variant_name, field_name);
                                self.add_var(
                                    tcx,
                                    child_name,
                                    field_ty,
                                    Some(name.clone()),
                                    VarKind::Field(rel),
                                    remaining_recursive_depth.map(|remaining| remaining - 1),
                                    in_array,
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
        tcx: &rustc_middle::ty::TyCtxt<'tcx>,
        name: String,
        adt_did: rustc_span::def_id::DefId,
        adt_generics: &[rustc_middle::ty::GenericArg<'tcx>],
        // parent: Option<String>,
        // var_kind: VarKind,
        remaining_recursive_depth: Option<usize>,
        in_array: bool,
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

            let len_name = format!("{}.length", name);
            self.variables.insert(
                len_name,
                VariableDecl::new(
                    VarKind::Field("length".to_string()),
                    DecType::Usize,
                    Some(name.clone()),
                    if in_array { 1 } else { 0 },
                    None,
                ),
            );

            let elem_ty = adt_generics[0]
                .as_type()
                .expect("Found Vec<_> with no specified element type");
            let elem_name = format!("{}[..]", name);
            self.add_var(
                tcx,
                elem_name,
                elem_ty,
                Some(name),
                VarKind::Array,
                remaining_recursive_depth.map(|remaining| remaining - 1),
                in_array,
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
            let pointee = format!("*{}", name);

            // FIXME: I don't think the varkind here should be Variable. Not sure what else
            // to do though, maybe keep it whatever it was previously?
            self.add_var(
                tcx,
                pointee,
                pointee_ty,
                Some(name),
                VarKind::Variable,
                remaining_recursive_depth.map(|remain| remain - 1),
                in_array,
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
