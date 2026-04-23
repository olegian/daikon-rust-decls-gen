use std::io::Write;

use crate::callbacks;

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
enum VarKind {
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
enum DecType {
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
    Str,
    /// Any aggregate type (struct / enum / tuple / array / slice / reference).
    /// the stored string is the user-facing declared-type rendering.
    Compound(String),
}

impl DecType {
    fn to_rep_type(&self) -> &'static str {
        match self {
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
            // char is utf-8 in Rust and doesn't fit cleanly into `int`; treat
            // both it and `str` as Java strings at the rep-type level.
            DecType::Char | DecType::Str => "java.lang.String",
            DecType::Compound(_) => "hashcode",
        }
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
            DecType::Compound(s) => return f.write_str(s),
        };
        f.write_str(s)
    }
}

impl DecType {
    fn from_ty<'b>(ty: rustc_middle::ty::Ty<'b>) -> Self {
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
struct VariableDecl {
    var_kind: VarKind,
    dec_type: DecType,
    enclosing_var: Option<String>,
    array: u8,
    comparability: Option<i64>,
}

impl std::fmt::Display for VariableDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  var-kind {}", self.var_kind)?;
        writeln!(f, "  dec-type {}", self.dec_type)?;
        writeln!(f, "  rep-type {}", self.dec_type.to_rep_type())?;
        if let Some(enc) = &self.enclosing_var {
            writeln!(f, "  enclosing-var {}", enc)?;
        }
        if self.array > 0 {
            writeln!(f, "  array {}", self.array)?;
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
pub struct ProgramPoint {
    ppt_type: ProgramPointType,
    variables: std::collections::HashMap<String, VariableDecl>,
    parents: std::collections::HashMap<String, (ParentRelationType, u64)>,
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
            VariableDecl {
                var_kind,
                dec_type: DecType::from_ty(ty),
                enclosing_var: parent,
                array: if in_array { 1 } else { 0 },
                comparability: None,
            },
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
                    VariableDecl {
                        var_kind: VarKind::Field("length".to_string()),
                        dec_type: DecType::Usize,
                        enclosing_var: Some(name.clone()),
                        array: if in_array { 1 } else { 0 },
                        comparability: None,
                    },
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

#[derive(Debug, Default)]
pub struct DeclsFile {
    ppts: std::collections::HashMap<String, ProgramPoint>,
}

impl std::fmt::Display for DeclsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "decl-version 2.0")?;
        writeln!(f, "input-language rust")?;
        writeln!(f, "var-comparability implicit")?;
        writeln!(f, "")?;

        for (ppt_name, ppt) in &self.ppts {
            writeln!(f, "ppt {ppt_name}")?;
            writeln!(f, "ppt-type {}", ppt.ppt_type)?;
            for (subexit_ppt, (relation_type, relation_id)) in &ppt.parents {
                writeln!(f, "parent {relation_type} {subexit_ppt} {relation_id}")?;
            }

            for (var_name, var_decl) in &ppt.variables {
                writeln!(f, "variable {var_name}")?;
                write!(f, "{}", var_decl)?;
            }

            writeln!(f, "")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum DeclsFileParseError {
    FileError(std::io::Error),
    BadHeader(&'static str),
    BadStructure(&'static str),
    // Would be really nice to make this identify which ppt was the offending one.
    MalformedPpt,
}

impl DeclsFile {
    /// Reads in and parses an existing decls file.
    pub fn from_decls_file(decls_file: &std::path::Path) -> Result<Self, DeclsFileParseError> {
        let content =
            std::fs::read_to_string(decls_file).map_err(|e| DeclsFileParseError::FileError(e))?;
        let mut lines = content
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .peekable();

        // Header
        if lines.next() != Some("decl-version 2.0") {
            return Err(DeclsFileParseError::BadHeader(
                "Input file is not of decls version 2.0",
            ));
        }
        if lines.next() != Some("input-language rust") {
            return Err(DeclsFileParseError::BadHeader(
                "Non-rust decls file provided as input",
            ));
        }
        // not checking comparibility. is that fine?

        // skip rest of header?
        while let Some(line) = lines.peek() && !line.starts_with("ppt") {
            lines.next();
        }

        let mut decls = DeclsFile::default();
        while let Some(line) = lines.next() {
            let ppt_name = line
                .strip_prefix("ppt ")
                .ok_or(DeclsFileParseError::BadStructure(
                    "did not find ppt tag where expected",
                ))?
                .to_string();

            let ppt_type_line = lines
                .next()
                .ok_or(DeclsFileParseError::MalformedPpt)?;
            let ppt_type_str = ppt_type_line
                .strip_prefix("ppt-type ")
                .ok_or(DeclsFileParseError::MalformedPpt)?;
            let ppt_type = match ppt_type_str {
                "enter" => ProgramPointType::Enter,
                "exit" => ProgramPointType::Exit,
                "subexit" => {
                    let id_str = ppt_name
                        .rsplit_once(":::EXIT")
                        .ok_or(DeclsFileParseError::MalformedPpt)?
                        .1;
                    ProgramPointType::ExitNN(
                        id_str
                            .parse()
                            .map_err(|_| DeclsFileParseError::MalformedPpt)?,
                    )
                }
                _ => return Err(DeclsFileParseError::MalformedPpt),
            };

            let mut parents = std::collections::HashMap::new();
            let mut variables = std::collections::HashMap::new();

            // Parent lines
            while let Some(peek) = lines.peek() {
                let Some(rest) = peek.strip_prefix("parent ") else {
                    break;
                };
                let mut parts = rest.splitn(3, ' ');
                let rel_str = parts.next().ok_or(DeclsFileParseError::MalformedPpt)?;
                let parent_name = parts
                    .next()
                    .ok_or(DeclsFileParseError::MalformedPpt)?
                    .to_string();
                let rel_id: u64 = parts
                    .next()
                    .ok_or(DeclsFileParseError::MalformedPpt)?
                    .parse()
                    .map_err(|_| DeclsFileParseError::MalformedPpt)?;
                let rel_type = match rel_str {
                    "parent" => ParentRelationType::Parent,
                    "enter-exit" => ParentRelationType::EnterExit,
                    "exit-exitnn" => ParentRelationType::ExitExitNN,
                    _ => return Err(DeclsFileParseError::MalformedPpt),
                };
                parents.insert(parent_name, (rel_type, rel_id));
                lines.next();
            }

            // Variable blocks
            while let Some(peek) = lines.peek() {
                let Some(var_name) = peek.strip_prefix("variable ") else {
                    break;
                };
                let var_name = var_name.to_string();
                lines.next();

                let mut var_kind: Option<VarKind> = None;
                let mut dec_type: Option<DecType> = None;
                let mut enclosing_var: Option<String> = None;
                let mut array: u8 = 0;
                let mut comparability: Option<i64> = None;

                while let Some(field_line) = lines.peek() {
                    let trimmed = field_line.trim_start();
                    if trimmed.starts_with("ppt ")
                        || trimmed.starts_with("variable ")
                        || trimmed.starts_with("parent ")
                    {
                        break;
                    }
                    lines.next();

                    if let Some(rest) = trimmed.strip_prefix("var-kind ") {
                        var_kind = Some(match rest {
                            "variable" => VarKind::Variable,
                            "array" => VarKind::Array,
                            "return" => VarKind::Return,
                            _ => {
                                if let Some(rel) = rest.strip_prefix("field ") {
                                    VarKind::Field(rel.to_string())
                                } else if let Some(rel) = rest.strip_prefix("function ") {
                                    VarKind::Function(rel.to_string())
                                } else {
                                    return Err(DeclsFileParseError::MalformedPpt);
                                }
                            }
                        });
                    } else if let Some(rest) = trimmed.strip_prefix("dec-type ") {
                        dec_type = Some(rest.into());
                    } else if trimmed.starts_with("rep-type ") {
                        // derived from dec-type, ignore
                    } else if let Some(rest) = trimmed.strip_prefix("enclosing-var ") {
                        enclosing_var = Some(rest.to_string());
                    } else if let Some(rest) = trimmed.strip_prefix("array ") {
                        array = rest.parse().map_err(|_| DeclsFileParseError::MalformedPpt)?;
                    } else if let Some(rest) = trimmed.strip_prefix("comparability ") {
                        let v: i64 = rest.parse().map_err(|_| DeclsFileParseError::MalformedPpt)?;
                        comparability = if v < 0 { None } else { Some(v) };
                    } else {
                        return Err(DeclsFileParseError::MalformedPpt);
                    }
                }

                variables.insert(
                    var_name,
                    VariableDecl {
                        var_kind: var_kind.ok_or(DeclsFileParseError::MalformedPpt)?,
                        dec_type: dec_type.ok_or(DeclsFileParseError::MalformedPpt)?,
                        enclosing_var,
                        array,
                        comparability,
                    },
                );
            }

            decls.ppts.insert(
                ppt_name,
                ProgramPoint {
                    ppt_type,
                    variables,
                    parents,
                },
            );
        }

        Ok(decls)
    }

    /// Compiles the crate identified by the `crate_root_file`,
    /// discovering all information required to write a decls file.
    pub fn from_source_file(crate_root_file: &std::path::Path) -> Self {
        let args = vec![
            "decls-gen".to_string(),
            crate_root_file.to_str().unwrap().to_string(),
        ];
        let mut cbs = callbacks::ConstructDecls::default();
        rustc_driver::run_compiler(&args, &mut cbs);

        cbs.into_decls_file()
    }

    /// Writes the information contained within self to a .decls file, in the
    /// proper format.
    pub fn write_to_file(self, file: &std::path::Path) -> std::io::Result<()> {
        let mut file =
            std::fs::File::create(file).expect("Unable to open output file for writing.");
        writeln!(file, "{}", self)
    }

    pub fn add_program_point(&mut self, name: String, ppt: ProgramPoint) {
        self.ppts.insert(name, ppt);
    }

    pub fn get_program_point_mut(&mut self, name: &str) -> Option<&mut ProgramPoint> {
        self.ppts.get_mut(name)
    }
}
