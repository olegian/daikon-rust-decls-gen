use crate::VarName;

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
// FIXME: it's used for pure functions. this might correspond to all const fn's
// defined on the type.
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
    Str,    // &str type
    String, // std::string::String type
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
    /// Any aggregate type (struct / enum / tuple / array / slice / reference).
    /// the stored string is the rust, user-facing, dec-type
    Compound(String),
}

impl DecType {
    pub fn to_rep_type(&self, array: Option<u8>) -> String {
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
            // char is utf-8 in Rust and doesn't fit cleanly into int.
            // treat both it and str as Java strings
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
        ty: &rustc_middle::ty::Ty<'tcx>,
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

/// The `constant` tag attached to a variable declaration.
///
/// `None` means no `constant` line is emitted.
/// `Uninit` for values that have been dropped by the time execution reaches
///   this program point.
/// `Boolean(repr)` is `"true"` or `"false"`.
/// `String(repr)` is either a quoted string literal ("\"hello\"") or
///   a single-quoted char literal (e.g. "'c'")
/// `Numeric(repr)` covers integer/float scalar values and array/slice lengths.
#[derive(Debug)]
pub enum Constant {
    None,
    Uninit,
    Boolean(String),
    String(String),
    Numeric(String),
}

impl Constant {
    pub fn is_some(&self) -> bool {
        !matches!(self, Constant::None)
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::None => Ok(()),
            Constant::Uninit => f.write_str("UNINITIALIZED"),
            Constant::Boolean(s) | Constant::String(s) | Constant::Numeric(s) => f.write_str(s),
        }
    }
}

impl std::convert::From<&str> for Constant {
    fn from(value: &str) -> Self {
        match value {
            "UNINITIALIZED" => Constant::Uninit,
            "true" | "false" => Constant::Boolean(value.to_string()),
            s if s.starts_with('"') || s.starts_with('\'') => Constant::String(s.to_string()),
            s => Constant::Numeric(s.to_string()),
        }
    }
}

/// A single variable declaration, as specified by section A.3.3 of the Daikon
/// Developer Documentation.
#[derive(Debug)]
pub struct VariableDecl {
    var_kind: VarKind,
    dec_type: DecType,
    enclosing_var: Option<VarName>,
    array: Option<u8>,
    constant: Constant,
    comparability: Option<u64>,
}

impl VariableDecl {
    pub fn new(var_kind: VarKind, dec_type: DecType) -> Self {
        Self {
            var_kind,
            dec_type,
            enclosing_var: None,
            array: None,
            comparability: None,
            constant: Constant::None,
        }
    }

    pub fn with_array(mut self, dim: Option<u8>) -> Self {
        self.array = dim;
        self
    }

    pub fn with_enclosing_var(mut self, enclosing_var: Option<VarName>) -> Self {
        self.enclosing_var = enclosing_var;
        self
    }

    pub fn with_constant(mut self, constant: Constant) -> Self {
        self.constant = constant;
        self
    }

    pub fn set_constant(&mut self, constant: Constant) {
        self.constant = constant;
    }

    pub fn with_comparability(mut self, comp: Option<u64>) -> Self {
        self.comparability = comp;
        self
    }

    pub fn set_comparability(&mut self, comp: Option<u64>) {
        self.comparability = comp;
    }

    pub fn get_comparability(&self) -> &Option<u64> {
        &self.comparability
    }

    pub fn get_dec_type(&self) -> &DecType {
        &self.dec_type
    }

    /// Returns true iff this declaration carries a `constant` tag of any kind.
    pub fn is_constant(&self) -> bool {
        self.constant.is_some()
    }

    /// Returns true iff this declaration's `constant` tag is the
    /// `UNINITIALIZED`, i.e. the value has been moved-out / dropped
    /// by the time execution reaches the enclosing program point.
    pub fn is_uninit(&self) -> bool {
        matches!(self.constant, Constant::Uninit)
    }

    /// Get the constant tag associated with this variable.
    pub fn constant(&self) -> &Constant {
        &self.constant
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
        if self.constant.is_some() {
            writeln!(f, "  constant {}", self.constant)?;
        }
        // kind of a gross cast,
        writeln!(f, "  comparability {}", self.comparability.map_or(-1, |comp| comp as i128))?;
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
