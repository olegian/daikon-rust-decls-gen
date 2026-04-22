use rustc_span::sym::Ok;

use crate::callbacks;
use std::io::Write;

#[derive(Debug)]
pub enum ProgramPointType {
    Enter,
    Exit(u64),
}

impl std::fmt::Display for ProgramPointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramPointType::Enter => f.write_str("enter"),
            ProgramPointType::Exit(_) => f.write_str("exit"),
        }
    }
}

#[derive(Debug)]
enum VarKind {
    Field {
        dec_type: DecType,
        enclosing_var: String,
        relative_name: String,
    },
    Function {
        enclosing_var: String,
        relative_name: String,
    },
    Array {
        dec_type: DecType,
        enclosing_var: String,
        dim: usize,
    },
    Variable {
        dec_type: DecType,
    },
    Return {
        dec_type: DecType,
    },
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

    Compound,
}

impl DecType {
    fn to_rep_type(&self) -> String {
        match self {
            DecType::U8 |
            DecType::U16 |
            DecType::U32 |
            DecType::U64 |
            DecType::U128 |
            DecType::Usize |
            DecType::I8 |
            DecType::I16 |
            DecType::I32 |
            DecType::I64 |
            DecType::I128 |
            DecType::Isize => "int".to_string(),
            DecType::F16 |
            DecType::F32 |
            DecType::F64 |
            DecType::F128 => "double".to_string(),
            DecType::Bool => "boolean".to_string(),
            // Could technically use char --> u8 --> int
            // but also char in rust is utf-8
            DecType::Char |
            DecType::Str => "java.lang.String".to_string(),
            DecType::Compound => "TEMPORARY::COMPOUND".to_string(),
        }
    }
}

impl std::fmt::Display for DecType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
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
            DecType::Compound => "TEMPORARY::COMPOUND",
        };

        f.write_str(s)
    }
}

impl<'a, 'b> From<&'a rustc_middle::ty::Ty<'b>> for DecType {
    fn from(value: &'a rustc_middle::ty::Ty<'b>) -> Self {
        match value.kind() {
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

            rustc_type_ir::TyKind::Adt(_, _)
            | rustc_type_ir::TyKind::Array(_, _)
            | rustc_type_ir::TyKind::Slice(_)
            | rustc_type_ir::TyKind::Tuple(_) => DecType::Compound,

            _ => {
                panic!("Unable to construct DecType from: {:?}", value)
            }
        }
    }
}

/// This is a representation of a single variable declaration as specified
/// by section A.3.3 of the Daikon Developer Documentation.
#[derive(Debug)]
struct VariableDecl {
    var_kind: VarKind,
    comparibility: Option<i64>,
}

impl std::fmt::Display for VariableDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.write_str("variable")
        match &self.var_kind {
            VarKind::Field {
                dec_type,
                enclosing_var,
                relative_name,
            } => {
                writeln!(f, "  var-kind field {}", relative_name)?;
                writeln!(f, "  dec-type {}", dec_type.to_string())?;
                writeln!(f, "  rep-type {}", dec_type.to_rep_type())?;
                writeln!(f, "  enclosing-var {}", enclosing_var)?;

                let comp = if let Some(comp) = self.comparibility {
                    comp
                } else {
                    -1
                };
                writeln!(f, "  comparability {}", comp)?;
            }
            VarKind::Function {
                enclosing_var,
                relative_name,
            } => todo!(),
            VarKind::Array {
                dec_type,
                enclosing_var,
                dim,
            } => {
                writeln!(f, "  var-kind array")?;
                writeln!(f, "  dec-type {}", dec_type.to_string())?;
                writeln!(f, "  rep-type {}", dec_type.to_rep_type())?;
                writeln!(f, "  array {}", if *dim > 0 { 1 } else { 0 })?;
                writeln!(f, "  enclosing-var {}", enclosing_var)?;

                let comp = if let Some(comp) = self.comparibility {
                    comp
                } else {
                    -1
                };
                writeln!(f, "  comparability {}", comp)?;
            }
            VarKind::Variable { dec_type } => {
                writeln!(f, "  var-kind variable")?;
                writeln!(f, "  dec-type {}", dec_type.to_string())?;
                writeln!(f, "  rep-type {}", dec_type.to_rep_type())?;

                let comp = if let Some(comp) = self.comparibility {
                    comp
                } else {
                    -1
                };
                writeln!(f, "  comparability {}", comp)?;
            },
            VarKind::Return { dec_type } => todo!(),
        }

        std::fmt::Result::Ok(())
    }
}

impl VariableDecl {
    pub fn new(var_kind: VarKind, comparibility: Option<i64>) -> Self {
        VariableDecl {
            var_kind,
            comparibility,
        }
    }

    pub fn set_comparibility(&mut self, abstract_type_id: i64) {
        self.comparibility = Some(abstract_type_id)
    }
}

#[derive(Debug)]
pub struct ProgramPoint {
    ppt_type: ProgramPointType,
    variables: std::collections::HashMap<String, VariableDecl>,
}

struct EnclosingVar(String, EnclosingKind);
enum EnclosingKind {
    None,
    Struct,
    Array(usize), // dim
}

impl EnclosingVar {
    fn empty() -> Self {
        EnclosingVar("".to_string(), EnclosingKind::None)
    }

    fn append_struct(&self, var_name: &str) -> Self {
        match self.1 {
            EnclosingKind::None => EnclosingVar(var_name.to_string(), EnclosingKind::Struct),
            EnclosingKind::Struct => EnclosingVar(
                format!("{}.{}", self.0, var_name.to_string()),
                EnclosingKind::Struct,
            ),
            EnclosingKind::Array(_) => EnclosingVar(
                format!("{}.{}", self.0, var_name.to_string()),
                EnclosingKind::Struct,
            ),
        }
    }

    fn append_array(&self, var_name: &str) -> Self {
        match self.1 {
            // brand new array
            EnclosingKind::None => EnclosingVar(var_name.to_string(), EnclosingKind::Array(1)),
            // brand new array, inside existing struct
            EnclosingKind::Struct => {
                EnclosingVar(format!("{}.{}", self.0, var_name), EnclosingKind::Array(1))
            }
            // array of higher dim, in other array
            EnclosingKind::Array(old_dim) => EnclosingVar(
                format!("{}{}", self.0, var_name),
                EnclosingKind::Array(old_dim + 1),
            ),
        }
    }

    fn with_leaf(&self, var_name: &str, dec_type: DecType) -> (String, VarKind) {
        match self.1 {
            EnclosingKind::None => (var_name.to_string(), VarKind::Variable { dec_type }),
            EnclosingKind::Struct => (
                format!("{}.{}", self.0, var_name),
                VarKind::Field {
                    dec_type,
                    enclosing_var: self.0.to_string(),
                    relative_name: var_name.to_string(),
                },
            ),
            EnclosingKind::Array(dim) => (
                format!("{}{}", self.0, var_name),
                VarKind::Array {
                    dec_type,
                    enclosing_var: self.0.to_string(),
                    dim: dim,
                },
            ),
        }
    }
}

impl ProgramPoint {
    pub fn enter(name: &str) -> (String, Self) {
        (
            format!("{name}:::ENTER"),
            Self {
                ppt_type: ProgramPointType::Enter,
                variables: std::collections::HashMap::new(),
            },
        )
    }

    pub fn include_fn_inputs<'a, 'b>(
        &mut self,
        tcx: &'a rustc_middle::ty::TyCtxt<'b>,
        inputs: impl Iterator<Item = (String, &'a rustc_middle::ty::Ty<'b>)>,
    ) {
        for (var_name, ty) in inputs.into_iter() {
            self.recursively_add_var_decls(tcx, EnclosingVar::empty(), var_name, ty);
        }
    }

    fn recursively_add_var_decls<'a, 'b>(
        &mut self,
        tcx: &'a rustc_middle::ty::TyCtxt<'b>,
        enclosing_var: EnclosingVar,
        current_var: String,
        ty: &'a rustc_middle::ty::Ty<'b>,
    ) {
        match ty.kind() {
            // Leaf types. These types do not contain any other variables inside of them,
            // and are therefore just added to the program point.
            rustc_type_ir::TyKind::Bool
            | rustc_type_ir::TyKind::Char
            | rustc_type_ir::TyKind::Int(_)
            | rustc_type_ir::TyKind::Uint(_)
            | rustc_type_ir::TyKind::Float(_)
            | rustc_type_ir::TyKind::Str => {
                let (var_name, var_kind) = enclosing_var.with_leaf(&current_var, DecType::from(ty));
                self.variables
                    .insert(var_name, VariableDecl::new(var_kind, None));
            }

            // Types which act as simple array-style collections of types.
            // man fuck this right now.
            rustc_type_ir::TyKind::Array(inner_ty, _) | rustc_type_ir::TyKind::Slice(inner_ty) => {
                // include array itself ...
                // ... include the length of the array ...
                let len_var_name = match enclosing_var.1 {
                    EnclosingKind::None => format!("{}.length", current_var),
                    EnclosingKind::Struct => format!("{}.{}.length", enclosing_var.0, current_var),
                    EnclosingKind::Array(_) => format!("{}{}.length", enclosing_var.0, current_var),
                };

                let len_var_kind = VarKind::Variable {
                    dec_type: DecType::Usize,
                };
                self.variables
                    .insert(len_var_name, VariableDecl::new(len_var_kind, None));

                // ... and whatever is stored in the array
                let next_enclosing = enclosing_var.append_array(&current_var);
                self.recursively_add_var_decls(tcx, next_enclosing, "[..]".to_string(), inner_ty);
            }

            rustc_type_ir::TyKind::Tuple(inner_tys) => todo!(),

            rustc_type_ir::TyKind::Ref(_, inner_ty, mutability) => {
                self.recursively_add_var_decls(tcx, enclosing_var, current_var, inner_ty);
            }

            rustc_type_ir::TyKind::Adt(adt_def, generics) => {
                if !adt_def.did().is_local() {
                    // type is foreign to crate.
                    // we special case a few types, collections and smart pointers.
                    // but otherwise ignore?
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

                            self.recursively_add_var_decls(
                                tcx,
                                enclosing_var.append_struct(&current_var),
                                field_name,
                                &field_ty,
                            );
                        }
                    }
                    rustc_middle::ty::AdtKind::Enum => {
                        for variant in adt_def.variants() {
                            let variant_name = variant.ident(*tcx).name.to_string();

                            match variant.ctor_kind() {
                                Some(_) => {
                                    // struct / tuple variant enum.
                                    // luckily, fields of tuples have index-names (e.g. "0", "1", ...).
                                    for field in &variant.fields {
                                        let field_name = field.ident(*tcx).name.to_string();
                                        let field_ty = tcx
                                            .type_of(field.did)
                                            .instantiate(*tcx, generics)
                                            .skip_normalization();

                                        self.recursively_add_var_decls(
                                            tcx,
                                            enclosing_var.append_struct(&format!(
                                                "{current_var}::{variant_name}"
                                            )),
                                            field_name,
                                            &field_ty,
                                        );
                                    }
                                }
                                None => {
                                    // unit variant
                                }
                            }
                        }
                    }
                    rustc_middle::ty::AdtKind::Union => {
                        panic!("Union parameter types are not supported.")
                    }
                }
            }

            _ => {
                // panic!("Encountered unsupported parameter type: {:?}", ty)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct DeclsFile {
    ppts: std::collections::HashMap<String, ProgramPoint>,
}

impl DeclsFile {
    /// Reads in and parses an existing decls file.
    pub fn from_decls_file(decls_file: &std::path::Path) -> Result<Self, ()> {
        todo!()
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
    pub fn write_to_file(self, file: &std::path::Path) {
        let mut file =
            std::fs::File::create(file).expect("Unable to open output file for writing.");
        writeln!(file, "decl-version 2.0").unwrap();
        writeln!(file, "input-language rust").unwrap();
        writeln!(file, "var-comparability implicit").unwrap();
        writeln!(file, "").unwrap();

        // FIXME: List implementors? should we specify things like Vec?
        // writeln!("ListImplementors")

        for (ppt_name, ppt) in self.ppts {
            writeln!(file, "ppt {ppt_name}").unwrap();
            writeln!(file, "ppt-type {}", ppt.ppt_type.to_string()).unwrap();
            // FIXME: finish: parent relation-type parent-ppt-name relation-id
            // writeln!(file, )

            for (var_name, var_decl) in ppt.variables {
                writeln!(file, "variable {var_name}").unwrap();
                write!(file, "{}", var_decl.to_string()).unwrap();
            }

            writeln!(file, "").unwrap();
        }
    }

    pub fn add_program_point(&mut self, name: String, ppt: ProgramPoint) {
        self.ppts.insert(name, ppt);
    }
}
