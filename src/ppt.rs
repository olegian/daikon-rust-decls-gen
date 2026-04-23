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
            false,
        );
    }

    /// Add all function inputs (formals) to this program point.
    pub fn include_fn_inputs<'a, 'b>(
        &mut self,
        tcx: &'a rustc_middle::ty::TyCtxt<'b>,
        inputs: impl Iterator<Item = (String, &'a rustc_middle::ty::Ty<'b>)>,
    ) {
        for (name, ty) in inputs {
            self.add_var(tcx, name, *ty, None, VarKind::Variable, false);
        }
    }

    /// Emit a record for the variable `name: ty`. If the type is a
    /// compound type, recursively emit records for its contained variables.
    ///
    /// `name` is the fully-qualified variable name (e.g. "arr[..].field").
    /// `parent` is the enclosing variable's fully-qualified name, or `None`.
    /// `var_kind` describes how this variable relates to its parent
    /// `parent_in_array` is true when any direct ancestor in the chain is an array
    fn add_var<'b>(
        &mut self,
        tcx: &rustc_middle::ty::TyCtxt<'b>,
        name: String,
        ty: rustc_middle::ty::Ty<'b>,
        parent: Option<String>,
        var_kind: VarKind,
        parent_in_array: bool,
    ) {
        // Peel all references to the current type
        let mut ty = ty;
        while let rustc_type_ir::TyKind::Ref(_, inner, _) = ty.kind() {
            ty = *inner;
        }

        // Any variable inside (or equal to) an array sequence has multiple
        // values, so it gets the <array 1> tag set, alongside any contained value.
        let in_array = parent_in_array || matches!(var_kind, VarKind::Array);

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
                self.add_var(tcx, elem_name, *inner, Some(name), VarKind::Array, in_array);
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
                        in_array,
                    );
                }
            }

            // Compound types
            rustc_type_ir::TyKind::Adt(adt_def, generics) => {
                if !adt_def.did().is_local() {
                    // Foreign ADTs (e.g. `Vec`, `Box`) need
                    // per-type special-casing. for now, just leave it
                    // as a hashcode, and stop recursing
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
}
