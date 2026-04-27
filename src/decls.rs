use std::io::Write;

use crate::{
    VarName, callbacks,
    fields::{DecType, ParentRelationType, ProgramPointType, VarKind, VariableDecl},
    ppt::ProgramPoint,
};

// include header information as well?
#[derive(Debug, Default)]
pub struct DeclsFile {
    ppts: std::collections::BTreeMap<String, ProgramPoint>,
}

pub const RETURN_VAR_NAME: &'static str = "return";
pub const FIELD_LENGTH: &'static str = "length";

// Creates a string representation of the entire decls file, in the .decls format.
impl<'a> std::fmt::Display for DeclsFile {
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
    // FIXME: would be really nice to make this identify which ppt was the offending one.
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
        while let Some(line) = lines.peek()
            && !line.starts_with("ppt")
        {
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

            let ppt_type_line = lines.next().ok_or(DeclsFileParseError::MalformedPpt)?;
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

            let mut parents = std::collections::BTreeMap::new();
            let mut variables = std::collections::BTreeMap::new();

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
                // probably deserves to be done via into(), with a From impl
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

                // included for all fields
                let mut var_kind: Option<VarKind> = None;
                let mut dec_type: Option<DecType> = None;
                let mut enclosing_var: Option<String> = None;
                let mut array: Option<u8> = None;
                let mut constant: Option<String> = None;
                let mut comparability: Option<i64> = None;

                while let Some(field_line) = lines.peek() {
                    let trimmed = field_line.trim_start();
                    if trimmed.starts_with("ppt ")
                        || trimmed.starts_with("variable ")
                        || trimmed.starts_with("parent ")
                    {
                        // end of current variable block, move onto next
                        break;
                    }

                    lines.next();

                    if let Some(rest) = trimmed.strip_prefix("var-kind ") {
                        // probably deserves to be done via into(), with a From impl
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
                        array = Some(
                            rest.parse::<u8>()
                                .map_err(|_| DeclsFileParseError::MalformedPpt)?,
                        );
                    } else if let Some(rest) = trimmed.strip_prefix("constant ") {
                        constant = Some(rest.to_string());
                    } else if let Some(rest) = trimmed.strip_prefix("comparability ") {
                        let v: i64 = rest
                            .parse()
                            .map_err(|_| DeclsFileParseError::MalformedPpt)?;
                        comparability = if v < 0 { None } else { Some(v) };
                    } else {
                        return Err(DeclsFileParseError::MalformedPpt);
                    }
                }

                let var_decl = VariableDecl::new(
                    var_kind.expect("Found a variable decl with no var_kind specified."),
                    dec_type.expect("Found a variable decl with no dec_type specified."),
                )
                .with_enclosing_var(enclosing_var.map(VarName::new))
                .with_array(array)
                .with_constant(constant)
                .with_comparability(comparability);

                variables.insert(var_name, var_decl);
            }

            decls
                .ppts
                .insert(ppt_name, ProgramPoint::new(ppt_type, variables, parents));
        }

        Ok(decls)
    }

    /// Compiles the crate identified by the `crate_root_file`,
    /// discovering all information required to write a decls file.
    /// `max_recursive_depth` specifies the maximum depth with which to expand variables
    /// of compound types. If None, then variables are expanded till a leaf type is found.
    /// If Some(0), output will only include program point information tags, with no variable
    /// declarations under any program point.
    pub fn from_source_file(
        crate_root_file: &std::path::Path,
        max_recursive_depth: Option<usize>,
    ) -> Self {
        let args = vec![
            "decls-gen".to_string(), // dummy value.
            crate_root_file.to_str().unwrap().to_string(),
        ];
        let mut cbs =
            callbacks::ConstructDecls::default().with_max_recursive_depth(max_recursive_depth);
        rustc_driver::run_compiler(&args, &mut cbs);

        cbs.into_decls_file()
    }

    /// Writes the information contained within self to a .decls file, in the
    /// proper format.
    pub fn write_to_file(&self, file: &std::path::Path) -> std::io::Result<()> {
        let mut file =
            std::fs::File::create(file).expect("Unable to open output file for writing.");
        writeln!(file, "{}", self)
    }

    pub fn add_program_point(&mut self, name: String, ppt: ProgramPoint) {
        self.ppts.insert(name, ppt);
    }
    /// Compute the fully qualified base ppt name for a function or method,
    /// without any `:::ENTER` / `:::EXIT` / `:::EXIT{N}` suffix.
    /// Can be used to query for all ENTER/EXIT/EXITNN ppts that correspond
    /// to this base name, via DeclsFile::ppts_for().
    pub fn ppt_base_name<'tcx>(
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        ldid: rustc_hir::def_id::LocalDefId,
    ) -> String {
        let file = file_name_of(tcx, tcx.def_span(ldid));
        let path = tcx.def_path_str(ldid);
        format!("{file}::{path}")
    }

    /// Compute the variable key used to look up a VariableDecl inside a
    /// ProgramPoint. Globals get a fully qualified <file_path>::<def_path>.
    /// formals/return get their normal name.
    pub fn var_name<'tcx>(tcx: rustc_middle::ty::TyCtxt<'tcx>, v: VarIdent) -> String {
        match v {
            VarIdent::Local(name) => name,
            VarIdent::Return => RETURN_VAR_NAME.to_string(),
            VarIdent::Global(did) => {
                let file = file_name_of(tcx, tcx.def_span(did));
                let path = tcx.def_path_str(did);
                format!("{file}::{path}")
            }
        }
    }

    /// get all program points (ENTER, EXIT, every EXITNN) for the given base ppt
    /// name.
    pub fn ppts_for(&self, base_ppt_name: &str) -> Vec<&ProgramPoint> {
        let lo = format!("{base_ppt_name}:::"); // can probably start with Enter? but eh
        let hi = format!("{base_ppt_name}:::~"); // ~ --> 0x7E will sort after any digit/letter
        self.ppts.range(lo..hi).map(|(_, p)| p).collect()
    }

    /// see DeclsFile::ppts_for
    pub fn ppts_for_mut(&mut self, base_ppt_name: &str) -> Vec<&mut ProgramPoint> {
        let lo = format!("{base_ppt_name}:::");
        let hi = format!("{base_ppt_name}:::~");
        self.ppts.range_mut(lo..hi).map(|(_, p)| p).collect()
    }

    pub fn get_program_point_mut(&mut self, name: &str) -> Option<&mut ProgramPoint> {
        self.ppts.get_mut(name)
    }

    pub fn get_program_point(&self, name: &str) -> Option<&ProgramPoint> {
        self.ppts.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&std::string::String, &ProgramPoint)> {
        self.ppts.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&std::string::String, &mut ProgramPoint)> {
        self.ppts.iter_mut()
    }
}

/// Identifies a variable for DeclsFile::var_name. Use Global
/// for any const or static item. Local for a formal parameter.
/// Return for the return value.
pub enum VarIdent {
    Local(String),
    Return,
    Global(rustc_hir::def_id::DefId),
}

/// Absolute path of the source file containing span.
pub fn file_name_of<'tcx>(tcx: rustc_middle::ty::TyCtxt<'tcx>, span: rustc_span::Span) -> String {
    let rustc_span::FileName::Real(rfn) = tcx.sess.source_map().span_to_filename(span) else {
        panic!("Attempting to get file name of span without an associated real file.");
    };
    // no extension? with extension?
    rfn.local_path().unwrap().display().to_string()
}
