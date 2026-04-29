use crate::{decls, fields::ParentRelationType, globals::Global, ppt::ProgramPoint};

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

        // Create all ENTER/EXIT PPTs, adding all formals/returns to
        // each appropriate one.
        // tcx.check_liveness(key)
        // for ldid in tcx.hir_body_owners() {
            // let results = tcx.mir_borrowck(ldid).unwrap();
            // let bck = rustc_borrowck::consumers::get_bodies_with_borrowck_facts(
            //     tcx,
            //     ldid,
            //     rustc_borrowck::consumers::ConsumerOptions::PoloniusOutputFacts,
            // );

            // rustc_mir_dataflow::move_paths::
            // rustc_middle::mir::
        // }

        // for ldid in tcx.hir_crate_items(()).definitions() {
        //     match tcx.hir_node_by_def_id(ldid) {
        //         rustc_hir::Node::Item(item) => match item.kind {
        //             rustc_hir::ItemKind::Fn { sig, ident, generics, body, has_body } => {
        //                 let bck = tcx.mir_borrowck(ldid).unwrap();
        //                 println!("{:#?}", bck);
        //             },
        //             _ => {}
        //         }
        //         _ => {}
        //     }
        // }

        // return rustc_driver::Compilation::Stop;

        let items = tcx.hir_crate_items(());
        for ldid in items.definitions() {
            let rustc_hir::Node::Item(item) = tcx.hir_node_by_def_id(ldid) else {
                continue;
            };
            match item.kind {
                rustc_hir::ItemKind::Fn { body, .. } => self.process_fn(compiler, tcx, ldid, body),
                rustc_hir::ItemKind::Impl(rustc_hir::Impl { items, .. }) => {
                    for assoc_item in items {
                        let method_ldid = assoc_item.owner_id.def_id;
                        // FIXME: only handling assoc functions for now; assoc consts
                        // may want to be surfaced in the decls file too.
                        if let rustc_hir::ImplItemKind::Fn(_, body_id) =
                            tcx.hir_expect_impl_item(method_ldid).kind
                        {
                            self.process_fn(compiler, tcx, method_ldid, body_id);
                        }
                    }
                }
                _ => {}
            }
        }

        // discover and add all globals to each ppt for which they are in scope,
        // evaluating them if they are a constant value.
        self.add_globals(tcx);

        rustc_driver::Compilation::Stop
    }
}

impl ConstructDecls {
    /// Discovers all const/static items in the crate and adds a var-decl for
    /// each one to every program point that has access to the const.
    /// Must be called after every ppt's subexits have already been constructed,
    /// to avoid MIR stealing issues.
    fn add_globals<'tcx>(&mut self, tcx: rustc_middle::ty::TyCtxt<'tcx>) {
        // collect all globals in the entire crate.
        // we will resolve which ones are visible where later.
        let items = tcx.hir_crate_items(());
        let mut globals: Vec<Global> = Vec::new();
        for ldid in items.definitions() {
            let node = tcx.hir_node_by_def_id(ldid);
            if let rustc_hir::Node::Item(item) = node {
                match item.kind {
                    rustc_hir::ItemKind::Static(_, _, _, _)
                    | rustc_hir::ItemKind::Const(_, _, _, _) => {
                        globals.push(Global::new(tcx, ldid));
                    }
                    _ => {}
                }
            }
        }

        // allows us to determine which globals are in scope at each ppt.
        let eff_vis = tcx.effective_visibilities(());

        // add a var-decl for each global to each ppt
        for (_, ppt) in self.decls.iter_mut() {
            ppt.add_globals(tcx, eff_vis, &globals, self.max_recursive_depth);
        }
    }

    /// Build all program points (ENTER, abstract EXIT, per-return-site EXITNN)
    /// for one function body, identified by its def-id and HIR `BodyId`.
    /// Used for both free fns and impl-method assoc fns; trait declarations
    /// are skipped at the call site.
    fn process_fn<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        ldid: rustc_hir::def_id::LocalDefId,
        body_id: rustc_hir::BodyId,
    ) {
        let base_ppt_name = decls::DeclsFile::ppt_base_name(tcx, ldid);

        let param_names: Vec<String> = tcx
            .hir_body(body_id)
            .params
            .iter()
            .map(|param| {
                param
                    .pat
                    .simple_ident()
                    .expect("Encountered input parameter with non-simple ident pat.")
                    .to_string()
            })
            .collect();

        // Liberate late-bound regions in the fn signature so downstream
        // queries (e.g. `type_is_copy_modulo_regions`) don't trip on
        // escaping bound vars in types like `&mut Formatter<'_>`.
        let sig = tcx.liberate_late_bound_regions(
            ldid.to_def_id(),
            tcx.fn_sig(ldid).instantiate_identity().skip_normalization(),
        );
        let input_tys: Vec<_> = sig.inputs().iter().copied().collect();
        let return_ty = sig.output();

        let enter_name =
            self.add_enter_ppt(tcx, &base_ppt_name, ldid, &param_names, &input_tys);
        self.add_exit_ppts(
            compiler,
            tcx,
            ldid,
            &base_ppt_name,
            enter_name,
            &param_names,
            &input_tys,
            return_ty,
        );
    }

    /// Adds the enter program point, given the provided input, to the decls file.
    fn add_enter_ppt<'tcx>(
        &mut self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        base_ppt_name: &str,
        local_def_id: rustc_hir::def_id::LocalDefId,
        param_names: &[String],
        input_tys: &[rustc_middle::ty::Ty<'tcx>],
    ) -> String {
        let (enter_name, enter_ppt) = ProgramPoint::enter(&base_ppt_name, local_def_id);
        let enter_ppt = enter_ppt.with_fn_inputs(
            tcx,
            param_names
                .iter()
                .cloned()
                .zip(input_tys.iter())
                .map(|(n, t)| (n, t, false)),
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
        enter_name: String,
        param_names: &[String],
        input_tys: &[rustc_middle::ty::Ty<'tcx>],
        return_ty: rustc_middle::ty::Ty<'tcx>,
    ) {
        // Tag each formal as `constant UNINITIALIZED` at exit iff the caller
        // had to give up ownership to make the call. From the caller's frame:
        //   - References (`&T`, `&mut T`) don't transfer ownership; the
        //     caller still holds the underlying value after the call.
        //   - Copy types are reproducible; the caller still has its copy.
        //   - Everything else (owned non-Copy) is consumed by the callee --
        //     moved into the return, dropped via Drop, or scope-ended -- and
        //     so is no longer meaningfully observable to the caller.
        // The classification is path-independent, so all subexits and the
        // abstract EXIT share the same uninit set.
        let typing_env = rustc_middle::ty::TypingEnv::post_analysis(tcx, ldid);
        let uninit: Vec<bool> = input_tys
            .iter()
            .map(|ty| {
                let is_ref = matches!(ty.kind(), rustc_type_ir::TyKind::Ref(..));
                !is_ref && !tcx.type_is_copy_modulo_regions(typing_env, *ty)
            })
            .collect();
        let inputs = || {
            param_names
                .iter()
                .cloned()
                .zip(input_tys.iter())
                .zip(uninit.iter().copied())
                .map(|((n, t), u)| (n, t, u))
        };

        // Abstract EXIT.
        let (exit_name, exit_ppt) = ProgramPoint::exit(base_ppt_name, ldid);
        let exit_ppt = exit_ppt
            .with_fn_inputs(tcx, inputs(), self.max_recursive_depth)
            .with_fn_return(tcx, return_ty, self.max_recursive_depth)
            .with_parent(
                enter_name,
                ParentRelationType::EnterExit,
                &mut self.next_parent_relation_id,
            );
        self.decls.add_program_point(exit_name.clone(), exit_ppt);

        // Walk MIR to discover return-sites for subexits, marked by assignment
        // to the return place (right before every return in mir_built, including
        // void/implicit ones) and Calls whose destination is the return
        // place. mir_built is a steal-query, so we collect spans and drop
        // the borrow before constructing further ppts.
        // FIXME: TailCall may also belong here; needs a fixture + MIR dump.
        let lines: Vec<u64> = {
            let mir = tcx.mir_built(ldid).borrow();
            let source_map = compiler.sess.source_map();
            let mut spans: Vec<rustc_span::Span> = Vec::new();
            for bb in mir.basic_blocks.iter() {
                for stmt in &bb.statements {
                    if let rustc_middle::mir::StatementKind::Assign(ref place_rvalue) = stmt.kind
                        && place_rvalue.0.local == rustc_middle::mir::RETURN_PLACE
                    {
                        spans.push(stmt.source_info.span);
                    }
                }
                if let rustc_middle::mir::TerminatorKind::Call { destination, .. } =
                    &bb.terminator().kind
                    && destination.local == rustc_middle::mir::RETURN_PLACE
                {
                    spans.push(bb.terminator().source_info.span);
                }
            }
            spans
                .iter()
                .map(|s| source_map.lookup_char_pos(s.source_callsite().lo()).line as u64)
                .collect()
        };

        let mut assigned = std::collections::HashSet::new();
        for line in lines {
            let mut candidate = line;
            while assigned.contains(&candidate) {
                candidate += 1;
            }
            assigned.insert(candidate);

            let (subexit_name, subexit_ppt) =
                ProgramPoint::subexit(base_ppt_name, ldid, candidate);
            let subexit_ppt = subexit_ppt
                .with_fn_inputs(tcx, inputs(), self.max_recursive_depth)
                .with_fn_return(tcx, return_ty, self.max_recursive_depth)
                .with_parent(
                    exit_name.clone(),
                    ParentRelationType::ExitExitNN,
                    &mut self.next_parent_relation_id,
                );

            self.decls.add_program_point(subexit_name, subexit_ppt);
        }
    }
}
