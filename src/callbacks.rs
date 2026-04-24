use crate::{decls, fields::ParentRelationType, ppt::ProgramPoint};

#[derive(Default)]
pub struct ConstructDecls {
    decls: decls::DeclsFile,

    // FIXME: not sure if the relation id needs to be unique for entire decls file
    // or if per program point, but i think it's safe to assume its over the
    // entire file.
    next_parent_relation_id: u64,

    // governs how deeply compound types are expanded into variable decls.
    // if zero, then goes the maximal depth, expanding compound types until
    // leaf types are found.
    max_recursive_depth: Option<usize>,
}

impl ConstructDecls {
    pub fn with_max_recursive_depth(mut self, max_recursive_depth: Option<usize>) -> Self {
        self.max_recursive_depth = max_recursive_depth;
        self
    }

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
        // Finding all instantiations of generic funcs
        // this might also find generic structs?
        // let res = tcx.collect_and_partition_mono_items(());
        // println!("{:#?}", res);
        // return rustc_driver::Compilation::Stop;

        // find crate that contains generic template?
        // Instance::upstream_monomorphization(&self, tcx)

        let items = tcx.hir_crate_items(());
        for ldid in items.definitions() {
            let node = tcx.hir_node_by_def_id(ldid);
            match node {
                rustc_hir::Node::Item(item) => {
                    match item.kind {
                        // Free functions
                        rustc_hir::ItemKind::Fn { ident, body, .. } => {
                            // Get name of ppts related to this function
                            let file_name = get_containing_file_name(compiler, item.span);
                            let base_ppt_name = format!("{}.{}", file_name, ident.as_str());

                            // extract relevant information regarding the function signature
                            let body = tcx.hir_body(body);
                            let param_names: Vec<String> = body
                                .params
                                .iter()
                                .map(|param| {
                                    param
                                        .pat
                                        .simple_ident()
                                        .expect(
                                            "Encountered input parameter with non-simple ident pat.",
                                        )
                                        .to_string()
                                })
                                .collect();
                            let sig = tcx.fn_sig(ldid).instantiate_identity().skip_binder();
                            let input_tys: Vec<_> = sig.inputs().iter().copied().collect();
                            let return_ty = sig.output();

                            // add enter, subexit, and exit ppts to the decls file
                            let enter_name =
                                self.add_enter_ppt(tcx, &base_ppt_name, &param_names, &input_tys);
                            self.add_exit_ppts(
                                compiler,
                                tcx,
                                ldid,
                                &base_ppt_name,
                                &enter_name,
                                &param_names,
                                &input_tys,
                                return_ty,
                            );
                        }

                        // Associated functions (inherent and trait impls)
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
                                let method_ldid = assoc_item.owner_id.def_id;
                                let owner = tcx.hir_expect_impl_item(method_ldid);

                                // for now, we only care about assoc functions. we probably
                                // should also do something with constants though
                                let rustc_hir::ImplItemKind::Fn(_sig, body_id) = owner.kind else {
                                    continue;
                                };

                                let base_ppt_name =
                                    format!("{}.{}.{}", file_name, self_ty, owner.ident.as_str());

                                let body = tcx.hir_body(body_id);
                                let param_names: Vec<String> = body
                                    .params
                                    .iter()
                                    .map(|param| {
                                        param
                                            .pat
                                            .simple_ident()
                                            .expect(
                                                "Encountered input parameter with non-simple ident pat.",
                                            )
                                            .to_string()
                                    })
                                    .collect();

                                let sig =
                                    tcx.fn_sig(method_ldid).instantiate_identity().skip_binder();
                                let input_tys: Vec<_> = sig.inputs().iter().copied().collect();
                                let return_ty = sig.output();

                                let enter_name = self.add_enter_ppt(
                                    tcx,
                                    &base_ppt_name,
                                    &param_names,
                                    &input_tys,
                                );
                                self.add_exit_ppts(
                                    compiler,
                                    tcx,
                                    method_ldid,
                                    &base_ppt_name,
                                    &enter_name,
                                    &param_names,
                                    &input_tys,
                                    return_ty,
                                );
                            }
                        }

                        rustc_hir::ItemKind::Trait(..) => {}

                        _ => {}
                    }
                }

                _ => {}
            };
        }

        rustc_driver::Compilation::Stop
    }
}

impl ConstructDecls {
    /// Adds the enter program point, given the provided input, to the decls file.
    fn add_enter_ppt<'tcx>(
        &mut self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        base_ppt_name: &str,
        param_names: &[String],
        input_tys: &[rustc_middle::ty::Ty<'tcx>],
    ) -> String {
        let (enter_name, mut enter_ppt) = ProgramPoint::enter(&base_ppt_name);
        enter_ppt.include_fn_inputs(
            &tcx,
            param_names.iter().cloned().zip(input_tys.iter()),
            self.max_recursive_depth,
        );
        self.decls.add_program_point(enter_name.clone(), enter_ppt);
        enter_name
    }

    /// Adds all subexit and exit program points, given the provided input, to the
    /// decls file. Assumes a corresponding enter site, with `enter_name` already exists.
    fn add_exit_ppts<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        ldid: rustc_span::def_id::LocalDefId,
        base_ppt_name: &str,
        enter_name: &str,
        param_names: &[String],
        input_tys: &[rustc_middle::ty::Ty<'tcx>],
        return_ty: rustc_middle::ty::Ty<'tcx>,
    ) {
        // get mir representation of the function of interest
        let mir = tcx.mir_built(ldid).borrow();
        let source_map = compiler.sess.source_map();

        let mut spans: Vec<rustc_span::Span> = Vec::new();
        for bb in mir.basic_blocks.iter() {
            // iterate through all statements of all basic blocks, looking for ...
            for stmt in &bb.statements {
                // ... places where we directly assign a value to the return place,
                // essentially %rax. These assignments happen right before all returns,
                // including for void functions (at least in the initially built mir,
                // before ANY OTHER OPTIMIZATIONS OCCUR -- that's why it's important
                // that we are using the mir_built query as opposed to any other one).
                // based on manual inspection using the `rustc -Z dump-mir` command),
                // so hopefully this doesn't change in the future?
                if let rustc_middle::mir::StatementKind::Assign(ref place_rvalue) = stmt.kind {
                    if place_rvalue.0.local == rustc_middle::mir::RETURN_PLACE {
                        spans.push(stmt.source_info.span);
                    }
                }
            }

            // ... invocations of functions that write to the return place.
            // note that TerminatorKind::Return is unnecessary for us, that would've been
            // caught by the assignment that came before it.
            if let rustc_middle::mir::TerminatorKind::Call { destination, .. } =
                &bb.terminator().kind
            {
                if destination.local == rustc_middle::mir::RETURN_PLACE {
                    spans.push(bb.terminator().source_info.span);
                }
            }

            // ... might also have to do something with TailCall? looks like the result of a TailCall
            // is always written to _0, which means it's a return point? write out
            // a tail-call test and dump out it's MIR to find out whats going on.
        }

        // Resolve each span to a source line via the source map
        let lines: Vec<u64> = spans
            .iter()
            .map(|s| source_map.lookup_char_pos(s.source_callsite().lo()).line as u64)
            .collect();

        // silly collision detection, if we ever encounter a duplicate line number, just bump up
        // until we find an id we are yet to use.
        // this probably makes the output slightly harder to understand, but should meet spec
        // requirements, keeping subexits with distinct ids.
        let mut assigned = std::collections::HashSet::new();
        let mut subexits = std::collections::HashSet::new();
        for line in lines {
            let mut candidate = line;
            while assigned.contains(&candidate) {
                candidate += 1;
            }
            assigned.insert(candidate);

            let (subexit_name, mut subexit_ppt) = ProgramPoint::subexit(base_ppt_name, candidate);
            subexit_ppt.include_fn_inputs(
                &tcx,
                param_names.iter().cloned().zip(input_tys.iter()),
                self.max_recursive_depth,
            );
            subexit_ppt.include_fn_return(&tcx, return_ty, self.max_recursive_depth);
            self.decls
                .add_program_point(subexit_name.clone(), subexit_ppt);
            subexits.insert(subexit_name);
        }

        // Now that subexits exist, create exit point.
        let (exit_name, mut exit_ppt) = ProgramPoint::exit(base_ppt_name);
        exit_ppt.include_fn_inputs(
            &tcx,
            param_names.iter().cloned().zip(input_tys.iter()),
            self.max_recursive_depth,
        );
        exit_ppt.include_fn_return(&tcx, return_ty, self.max_recursive_depth);

        for subexit in subexits.into_iter() {
            exit_ppt.add_parent(
                subexit,
                ParentRelationType::ExitExitNN,
                self.next_parent_relation_id,
            );
            self.next_parent_relation_id += 1;
        }
        self.decls.add_program_point(exit_name.clone(), exit_ppt);

        // edit enter point to include exit point parent field
        self.decls
            .get_program_point_mut(&enter_name)
            .expect(
                "Attempting to create exit point without first making corresponding enter point.",
            )
            .add_parent(
                exit_name,
                ParentRelationType::EnterExit,
                self.next_parent_relation_id,
            );
        self.next_parent_relation_id += 1;
    }
}

/// Returns the absolute path to the file which contains the input span.
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
