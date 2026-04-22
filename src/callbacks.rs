use crate::decls;

#[derive(Default)]
pub struct ConstructDecls {
    decls: decls::DeclsFile,
}

impl ConstructDecls {
    pub fn into_decls_file(self) -> decls::DeclsFile {
        self.decls
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
        // find globals first? so that we know to add var decls for them to all sites.

        let items = tcx.hir_crate_items(());
        for ldid in items.definitions() {
            let node = tcx.hir_node_by_def_id(ldid);
            match node {
                rustc_hir::Node::Item(item) => {
                    match item.kind {
                        // Add ENTER/EXIT sites for all free functions.
                        rustc_hir::ItemKind::Fn {
                            sig,
                            ident,
                            generics,
                            body,
                            has_body,
                        } => {
                            let file_name = get_containing_file_name(compiler, item.span);
                            let ppt_name = format!("{}.{}", file_name, ident.as_str());
                            let (ppt_name, mut enter_ppt) = decls::ProgramPoint::enter(&ppt_name);

                            let body = tcx.hir_body(body);
                            let param_names = body.params.iter().map(|param| {
                                param
                                    .pat
                                    .simple_ident()
                                    .expect(
                                        "Encountered input parameter with non-simple ident pat.",
                                    )
                                    .to_string()
                            });

                            // consider finding all instantiations of this func, and then create separate
                            // related sites for all concrete functions, then a site which represents the LUB
                            // of all concrete sites to represent the generic site.
                            // benefits of querying the MIR..
                            let sig = tcx.fn_sig(ldid).instantiate_identity().skip_binder();
                            let inputs = param_names.zip(sig.inputs());
                            enter_ppt.include_fn_inputs(&tcx, inputs);

                            self.decls.add_program_point(ppt_name, enter_ppt);
                        }



                        // Add ENTER/EXIT sites for all methods, including impls of traits
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

                        

                        // take notes of all default impls?
                        // Add ENTER/EXIT sites for all default functions of traits
                        rustc_hir::ItemKind::Trait(..) => {}

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
