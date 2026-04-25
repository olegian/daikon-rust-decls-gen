#[derive(Debug)]
pub enum ProgramPointType {
    Enter,
    Exit,
    ExitNN(u64),
}

impl std::fmt::Display for ProgramPointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramPointType::Enter => f.write_str("enter"),
            ProgramPointType::ExitNN(_) => f.write_str("subexit"),
            ProgramPointType::Exit => f.write_str("exit"),
        }
    }
}

/// The `var-kind <...>` line for a variable declaration
///
/// `Variable` is used for top-level variables (e.g. function parameters).
/// `Field(rel)` is used for any named sub-variable of a compound (struct field,
/// enum variant field, tuple field, `.length`). The string is the relative name
/// written after `var-kind field`.
/// `Array` is used for the `[..]` sequence of an array-like variable.
/// `Return` is used for return values.
/// `Function` is used for ??????????????
#[derive(Debug)]
pub enum VarKind {
    Variable,
    Field(String),
    Array,
    Return,
    Function(String),
}

impl std::fmt::Display for VarKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarKind::Variable => f.write_str("variable"),
            VarKind::Field(rel) => write!(f, "field {}", rel),
            VarKind::Array => f.write_str("array"),
            VarKind::Return => f.write_str("return"),
            VarKind::Function(rel) => write!(f, "function {}", rel),
        }
    }
}

#[derive(Debug)]
pub enum DecType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    F16,
    F32,
    F64,
    F128,
    Bool,
    Char,
    Str,    // &str type
    String, // std::string::String type
    /// Any aggregate type (struct / enum / tuple / array / slice / reference).
    /// the stored string is the rust, user-facing, dec-type
    Compound(String),
}

impl DecType {
    fn to_rep_type(&self, array: Option<u8>) -> String {
        let base = match self {
            DecType::U8
            | DecType::U16
            | DecType::U32
            | DecType::U64
            | DecType::U128
            | DecType::Usize
            | DecType::I8
            | DecType::I16
            | DecType::I32
            | DecType::I64
            | DecType::I128
            | DecType::Isize => "int",
            DecType::F16 | DecType::F32 | DecType::F64 | DecType::F128 => "double",
            DecType::Bool => "boolean",
            // char is utf-8 in Rust and doesn't fit cleanly into `int`.
            // treat both it and `str` as Java strings
            DecType::Char | DecType::Str | DecType::String => "java.lang.String",
            DecType::Compound(_) => "hashcode",
        };

        let suffix = if let Some(array) = array
            && array > 0
        {
            "[]"
        } else {
            ""
        };

        format!("{}{}", base, suffix)
    }
}

impl std::fmt::Display for DecType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = match self {
            DecType::U8 => "u8",
            DecType::U16 => "u16",
            DecType::U32 => "u32",
            DecType::U64 => "u64",
            DecType::U128 => "u128",
            DecType::Usize => "usize",
            DecType::I8 => "i8",
            DecType::I16 => "i16",
            DecType::I32 => "i32",
            DecType::I64 => "i64",
            DecType::I128 => "i128",
            DecType::Isize => "isize",
            DecType::F16 => "f16",
            DecType::F32 => "f32",
            DecType::F64 => "f64",
            DecType::F128 => "f128",
            DecType::Bool => "bool",
            DecType::Char => "char",
            DecType::Str => "str",
            DecType::String => "std::string::String",
            DecType::Compound(s) => return f.write_str(s),
        };
        f.write_str(s)
    }
}

impl DecType {
    pub fn from_ty<'tcx>(
        tcx: &rustc_middle::ty::TyCtxt<'tcx>,
        ty: rustc_middle::ty::Ty<'tcx>,
    ) -> Self {
        match ty.kind() {
            rustc_type_ir::TyKind::Bool => DecType::Bool,
            rustc_type_ir::TyKind::Char => DecType::Char,
            rustc_type_ir::TyKind::Str => DecType::Str,

            rustc_type_ir::TyKind::Int(int_ty) => match int_ty {
                rustc_ast::IntTy::Isize => DecType::Isize,
                rustc_ast::IntTy::I8 => DecType::I8,
                rustc_ast::IntTy::I16 => DecType::I16,
                rustc_ast::IntTy::I32 => DecType::I32,
                rustc_ast::IntTy::I64 => DecType::I64,
                rustc_ast::IntTy::I128 => DecType::I128,
            },

            rustc_type_ir::TyKind::Uint(uint_ty) => match uint_ty {
                rustc_ast::UintTy::Usize => DecType::Usize,
                rustc_ast::UintTy::U8 => DecType::U8,
                rustc_ast::UintTy::U16 => DecType::U16,
                rustc_ast::UintTy::U32 => DecType::U32,
                rustc_ast::UintTy::U64 => DecType::U64,
                rustc_ast::UintTy::U128 => DecType::U128,
            },

            rustc_type_ir::TyKind::Float(float_ty) => match float_ty {
                rustc_ast::FloatTy::F16 => DecType::F16,
                rustc_ast::FloatTy::F32 => DecType::F32,
                rustc_ast::FloatTy::F64 => DecType::F64,
                rustc_ast::FloatTy::F128 => DecType::F128,
            },

            rustc_type_ir::TyKind::Adt(adt_def, _) => {
                // String has to be recognized as a DecType::String.
                if adt_def.did()
                    == tcx
                        .lang_items()
                        .string()
                        .expect("Unable to find def id of std::string::String type")
                {
                    DecType::String
                } else {
                    DecType::Compound(ty.to_string())
                }
            }

            // Everything else is an aggregate and prints with hashcode rep-type.
            _ => DecType::Compound(ty.to_string()),
        }
    }
}

impl std::convert::From<&str> for DecType {
    fn from(value: &str) -> Self {
        match value {
            "u8" => DecType::U8,
            "u16" => DecType::U16,
            "u32" => DecType::U32,
            "u64" => DecType::U64,
            "u128" => DecType::U128,
            "usize" => DecType::Usize,
            "i8" => DecType::I8,
            "i16" => DecType::I16,
            "i32" => DecType::I32,
            "i64" => DecType::I64,
            "i128" => DecType::I128,
            "isize" => DecType::Isize,
            "f16" => DecType::F16,
            "f32" => DecType::F32,
            "f64" => DecType::F64,
            "f128" => DecType::F128,
            "bool" => DecType::Bool,
            "char" => DecType::Char,
            "str" => DecType::Str,

            // kinda dangerous to assume that all other strings are compound types...
            _ => DecType::Compound(value.to_string()),
        }
    }
}

/// A single variable declaration, as specified by section A.3.3 of the Daikon
/// Developer Documentation.
#[derive(Debug)]
pub struct VariableDecl {
    var_kind: VarKind,
    dec_type: DecType,
    enclosing_var: Option<String>,
    array: Option<u8>,
    constant: Option<String>,
    comparability: Option<i64>,
}

impl VariableDecl {
    pub fn new(var_kind: VarKind, dec_type: DecType) -> Self {
        Self {
            var_kind,
            dec_type,
            enclosing_var: None,
            array: None,
            comparability: None,
            constant: None,
        }
    }

    pub fn with_array(mut self, dim: Option<u8>) -> Self {
        self.array = dim;
        self
    }

    pub fn with_enclosing_var(mut self, enclosing_var: Option<String>) -> Self {
        self.enclosing_var = enclosing_var;
        self
    }

    pub fn with_constant(mut self, value_repr: Option<String>) -> Self {
        self.constant = value_repr;
        self
    }

    pub fn set_constant(&mut self, value_repr: Option<String>) {
        self.constant = value_repr;
    }

    pub fn with_comparability(mut self, comp: Option<i64>) -> Self {
        self.comparability = comp;
        self
    }
}

impl std::fmt::Display for VariableDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  var-kind {}", self.var_kind)?;
        writeln!(f, "  dec-type {}", self.dec_type)?;
        writeln!(f, "  rep-type {}", self.dec_type.to_rep_type(self.array))?;
        if let Some(enc) = &self.enclosing_var {
            writeln!(f, "  enclosing-var {}", enc)?;
        }
        if let Some(dim) = self.array {
            writeln!(f, "  array {}", dim)?;
        }
        if let Some(constant) = &self.constant {
            writeln!(f, "  constant {}", constant)?;
        }
        writeln!(f, "  comparability {}", self.comparability.unwrap_or(-1))?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ParentRelationType {
    Parent,
    EnterExit,
    ExitExitNN,
}

impl std::fmt::Display for ParentRelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParentRelationType::Parent => write!(f, "parent"),
            ParentRelationType::EnterExit => write!(f, "enter-exit"),
            ParentRelationType::ExitExitNN => write!(f, "exit-exitnn"),
        }
    }
}

#[derive(Debug)]
pub struct Global {
    pub name: String,
    pub ldid: rustc_span::def_id::LocalDefId,
}

impl<'a> Global {
    pub fn new(
        ldid: rustc_span::def_id::LocalDefId,
        file_name: String,
        ident: rustc_span::Ident,
    ) -> Global {
        let name = format!("{}.{}", file_name, ident.as_str());
        Self { name, ldid }
    }

    pub fn did(&self) -> rustc_hir::def_id::DefId {
        self.ldid.to_def_id()
    }

    /// Compile-time-evaluate this global and return a string repr of its value,
    /// or `None` if it isn't eligible for the `constant` tag (mutable static)
    /// or its value can't be represented as a single scalar (e.g. aggregate
    /// consts, slice consts, or any case the const-eval machinery can't
    /// reduce here).
    pub fn evaluate<'tcx>(&self, tcx: rustc_middle::ty::TyCtxt<'tcx>) -> Option<String> {
        // `const` items and immutable `static` items have fixed values for the
        // lifetime of the program; `static mut` items don't. The two cases also
        // require different rustc query entry points: `const_eval_global_id`
        // asserts the def-id is *not* a static, so statics must go through
        // `eval_static_initializer` and have a scalar read out of the resulting
        // allocation.
        let did = self.ldid.to_def_id();
        let is_const = match tcx.hir_node_by_def_id(self.ldid) {
            rustc_hir::Node::Item(item) => match item.kind {
                rustc_hir::ItemKind::Const(..) => true,
                rustc_hir::ItemKind::Static(rustc_ast::Mutability::Not, ..) => false,
                _ => return None,
            },
            _ => return None,
        };

        // Anything with hashcode rep-type (pointers, references, arrays-by-value,
        // structs/enums) gets a placeholder for now — rustc's raw scalar repr
        // (e.g. "pointer to alloc117<imm>") isn't a stable or meaningful value
        // for Daikon, and we don't yet decompose aggregates per-leaf.
        let ty = tcx
            .type_of(self.ldid)
            .instantiate_identity()
            .skip_normalization();
        if matches!(DecType::from_ty(&tcx, ty), DecType::Compound(_)) {
            return Some("PTR_TYPE".to_string());
        }

        if is_const {
            let ty_env = rustc_middle::ty::TypingEnv::fully_monomorphized();
            let instance = rustc_middle::ty::Instance::mono(tcx, did);
            let gid = rustc_middle::mir::interpret::GlobalId {
                instance,
                promoted: None,
            };
            match tcx.const_eval_global_id(ty_env, gid, rustc_span::DUMMY_SP) {
                Ok(rustc_middle::mir::ConstValue::Scalar(scalar)) => Some(scalar.to_string()),
                _ => None,
            }
        } else {
            let alloc = tcx.eval_static_initializer(did).ok()?;
            let inner = alloc.inner();
            let range = rustc_middle::mir::interpret::alloc_range(
                rustc_abi::Size::ZERO,
                inner.size(),
            );
            let scalar = inner.read_scalar(&tcx, range, false).ok()?;
            Some(scalar.to_string())
        }
    }

    #[allow(dead_code)]
    fn __evaluate__(&self) -> String {
        todo!()
        // theres a lot of  const_eval* functions that all do slightly different things.
        // based off the docs, the potentailly interesting ones are:
        //   const_eval_resolve(...) --> evaluate constant, resolving generic types as necessary.
        //     this call can fail, if the resolved generics are still "too generic". Not sure if
        //     constants like this can appear in the constants we are evaluating. maybe through
        //     evaluating a const function that itself has a generic param as input?
        //   const_eval_global_id(...) --> seems to be the most generic one? just evals const?
        //   const_eval_instance(...) --> same as above, but with some default (?) parameters.
        // based off https://rustc-dev-guide.rust-lang.org/const-eval.html, choosing _global_id().

        // ANOTHER THING: it would be great to eval constants early, so that we do not need
        // to recompute any of them when they are added to each ppt. With that said, evaluating
        // here would mean it happens before any subexit ppts get created, which rely on the mir_built
        // query. Executing this stuff, steals the result of mir_built however, which means this
        // evaluation HAS TO BE DONE AFTER CREATING SUBEXITS, most likely whenever the file
        // is actually being written then? Luckily query system should cache result, so
        // "recomputing" constant value should actually be really quick and simple.

        // let ty = tcx
        //     .type_of(ldid)
        //     .instantiate_identity()
        //     .skip_normalization();

        // let did = ldid.to_def_id();
        // let ty_env = rustc_middle::ty::TypingEnv::fully_monomorphized();
        // let instance = rustc_middle::ty::Instance::mono(tcx, did);
        // let gid = rustc_middle::mir::interpret::GlobalId {
        //     instance,
        //     promoted: None,
        // };

        // let value_repr = match tcx.const_eval_global_id(ty_env, gid, rustc_span::DUMMY_SP) {
        //     Ok(const_val) => {
        //         let a = match const_val {
        //             rustc_middle::mir::ConstValue::Scalar(scalar) => Some(scalar.to_string()),
        //             rustc_middle::mir::ConstValue::ZeroSized => {
        //                 // probably have const_var return an Option/Result instead at some point.
        //                 // ZSTs probably shouldn't be included in the decls file anyways.
        //                 panic!("ZSTs as const globals are unsupported.")
        //             }
        //             rustc_middle::mir::ConstValue::Slice { alloc_id, meta } => {
        //                 // FIXME:
        //                 // it's technically possible to peak at the allocation itself
        //                 // using tcx.global_alloc(alloc_id). but at this point, we are working
        //                 // with raw memory, i'd have to extract offsets to refer to meaningful values?
        //                 // and generate the tag for the constant tag.
        //                 // Once that's decided, change type of value_repr accordingly?
        //                 match tcx.global_alloc(alloc_id) {
        //                     rustc_const_eval::interpret::GlobalAlloc::Function { instance } => {}
        //                     rustc_const_eval::interpret::GlobalAlloc::Memory(const_allocation) => {}
        //                     rustc_const_eval::interpret::GlobalAlloc::TypeId { ty } => {
        //                         panic!("Type constant in const variable?")
        //                     }
        //                     rustc_const_eval::interpret::GlobalAlloc::Static(def_id) => {
        //                         panic!("Cannot determine value of lazy allocation at compile time.")
        //                     }
        //                     rustc_const_eval::interpret::GlobalAlloc::VTable(ty, raw_list) => {
        //                         panic!("VTable constant found in const variable?")
        //                     }
        //                 }

        //                 None
        //             }
        //             rustc_middle::mir::ConstValue::Indirect { alloc_id, offset } => {
        //                 // FIXME: as above.
        //                 match tcx.global_alloc(alloc_id) {
        //                     rustc_const_eval::interpret::GlobalAlloc::Function { instance } => {}
        //                     rustc_const_eval::interpret::GlobalAlloc::Memory(const_allocation) => {}
        //                     rustc_const_eval::interpret::GlobalAlloc::VTable(ty, raw_list) => {
        //                         panic!("VTable constant found in const variable?")
        //                     }
        //                     rustc_const_eval::interpret::GlobalAlloc::Static(def_id) => {
        //                         panic!("Cannot determine value of lazy allocation at compile time.")
        //                     }
        //                     rustc_const_eval::interpret::GlobalAlloc::TypeId { ty } => {
        //                         panic!("Type constant in const variable?")
        //                     }
        //                 }

        //                 None
        //             }
        //         };
        //         a
        //     }
        //     Err(e) => {
        //         panic!("Unable to evaluate constant {const_name}: {e:?}");
        //     }
        // };
    }
}
