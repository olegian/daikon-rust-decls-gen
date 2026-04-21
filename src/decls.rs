use crate::callbacks;

pub enum ProgramPointType {
    Enter,
    Exit,
}

enum RepType {
    Prim(String),
    PrimArray(String),
    HashCodeArray(String),
    HashCodeStruct(String),
    Skip,
}

impl RepType {
    pub fn from_ty(ty: &rustc_hir::Ty) -> Self {
        todo!();
    }
}

struct VariableDecl {
    name: String,
    rep_type: RepType, // compute dec_type from this?
}

impl VariableDecl {
    pub fn new(name: String, rep_type: RepType) -> Self {
        VariableDecl { name, rep_type }
    }
}

pub struct ProgramPoint {
    name: String,
    ppt_type: ProgramPointType,
    variables: Vec<VariableDecl>,
}

impl ProgramPoint {
    pub fn empty(name: String, ppt_type: ProgramPointType) -> Self {
        ProgramPoint {
            name,
            ppt_type,
            variables: Vec::new(),
        }
    }

    pub fn include_fn_formals(
        &mut self,
        tcx: &rustc_middle::ty::TyCtxt,
        body: &rustc_hir::Body,
        decls: &rustc_hir::FnDecl,
    ) {
        // I'm mostly sure that these will always be in a corresponding order...
        for (param, ty) in body.params.iter().zip(decls.inputs) {
            let var_name = param
                .pat
                .simple_ident()
                .expect("Encountered input parameter with non-simple ident pat.")
                .to_string();
            let rep_type = RepType::from_ty(ty);

            self.variables.push(VariableDecl::new(var_name, rep_type));
        }
    }
}

pub struct DeclsFile {}

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

        println!("PASSING: {:?}", args);

        let mut cbs = callbacks::ConstructDecls::default();
        rustc_driver::run_compiler(&args, &mut cbs);

        cbs.into_decls_file()
    }

    /// Writes the information contained within self to a .decls file, in the
    /// proper format.
    pub fn write_to_file(self, file: &std::path::Path) {
        todo!()
    }
}
