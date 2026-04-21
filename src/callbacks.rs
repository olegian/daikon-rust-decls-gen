use crate::decls;

#[derive(Default)]
pub struct ConstructDecls {
    ppts: Vec<decls::ProgramPoint>
}

impl ConstructDecls {
    pub fn into_decls_file(self) -> decls::DeclsFile {
        todo!()
    }
}

impl rustc_driver::Callbacks for ConstructDecls {
    fn config(&mut self, _config: &mut rustc_interface::interface::Config) {}

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _krate: &mut rustc_ast::Crate,
    ) -> rustc_driver::Compilation {
        rustc_driver::Compilation::Continue
    }

    fn after_expansion<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        let items = tcx.hir_crate_items(());
        for ldid in items.definitions() {
            let node = tcx.hir_node_by_def_id(ldid);
            match node {
                rustc_hir::Node::Item(item) => {
                    match item.kind {
                        rustc_hir::ItemKind::Fn {
                            sig,
                            ident,
                            generics,
                            body,
                            has_body,
                        } => {
                            let file_name = get_containing_file_name(compiler, item.span);
                            let ppt_name = format!("{}.{}", file_name, ident.as_str());
                            let body = tcx.hir_body(body);

                            let mut enter_ppt = decls::ProgramPoint::empty(
                                ppt_name.clone(),
                                decls::ProgramPointType::Enter,
                            );
                            enter_ppt.include_fn_formals(&tcx, body, sig.decl);

                            self.ppts.push(enter_ppt);
                        }
                        rustc_hir::ItemKind::Enum(ident, generics, enum_def) => {
                            let file_name = get_containing_file_name(compiler, item.span);
                            let ppt_name = format!("{}.{}", file_name, ident.as_str());
                        }
                        rustc_hir::ItemKind::Struct(ident, generics, variant_data) => {
                            let file_name = get_containing_file_name(compiler, item.span);
                            let ppt_name = format!("{}.{}", file_name, ident.as_str());
                        }

                        rustc_hir::ItemKind::Impl(rustc_hir::Impl { self_ty, items, .. }) => {
                            let file_name = get_containing_file_name(compiler, item.span);
                            let rustc_hir::TyKind::Path(rustc_hir::QPath::Resolved(_, path)) =
                                self_ty.kind
                            else {
                                panic!("Encountered impl block with non-Path kind self ty");
                            };

                            let self_ty = path
                                .segments
                                .iter()
                                .map(|seg| seg.ident.as_str())
                                .collect::<Vec<_>>()
                                .join(".");
                            for assoc_item in items {
                                let owner = tcx.hir_expect_impl_item(assoc_item.owner_id.def_id);
                                if let rustc_hir::ImplItemKind::Fn(sig, body_id) = owner.kind {
                                    let ppt_name = format!(
                                        "{}.{}.{}",
                                        file_name,
                                        self_ty,
                                        owner.ident.as_str()
                                    );
                                }
                            }
                        }

                        _ => {} // ignore all the other item kinds
                    }
                }

                _ => {} // ignore the rest of the node kinds
            };
        }

        // No reason to continue compilation after collecting all necessary information
        rustc_driver::Compilation::Stop
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        rustc_driver::Compilation::Stop
    }
}

fn get_containing_file_name(
    compiler: &rustc_interface::interface::Compiler,
    span: rustc_span::Span,
) -> String {
    let rustc_span::FileName::Real(rfn) = compiler.sess.source_map().span_to_filename(span) else {
        panic!("Attempting to get file name of span without an associated real file.");
    };
    let file_path = rfn.local_path().unwrap().with_extension("");
    file_path.display().to_string()
}
