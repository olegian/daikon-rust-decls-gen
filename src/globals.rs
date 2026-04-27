/// Represents a global, defined somewhere in the crate,
/// with a unique name, and ldid.
#[derive(Debug)]
pub struct Global {
    pub name: String,
    pub ldid: rustc_span::def_id::LocalDefId,
}

impl<'a> Global {
    pub fn new<'tcx>(
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        ldid: rustc_span::def_id::LocalDefId,
    ) -> Global {
        let name = crate::decls::DeclsFile::var_name(
            tcx,
            crate::decls::VarIdent::Global(ldid.to_def_id()),
        );
        Self { name, ldid }
    }

    pub fn did(&self) -> rustc_hir::def_id::DefId {
        self.ldid.to_def_id()
    }

    /// Build the initial ConstSource for this global, which can be recursively
    /// expanded mirroring the globals type. This let's us compute the actual values of
    /// constants recursively, by appropriately indexing/offsetting into a buffer,
    /// or reading from it when we encounter a scalar value.
    ///
    /// Returns None for globals not eligible for a constant tag in the decls file
    /// (mutable statics) or globals whose init value can't be evaluated.
    pub fn const_source<'tcx>(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ) -> Option<ConstSource<'tcx>> {
        let did = self.ldid.to_def_id();
        match tcx.hir_node_by_def_id(self.ldid) {
            rustc_hir::Node::Item(item) => match item.kind {
                rustc_hir::ItemKind::Const(..) => {
                    // run constant evalution:
                    let ty_env = rustc_middle::ty::TypingEnv::fully_monomorphized();
                    let instance = rustc_middle::ty::Instance::mono(tcx, did);
                    let gid = rustc_middle::mir::interpret::GlobalId {
                        instance,
                        promoted: None,
                    };
                    match tcx
                        .const_eval_global_id(ty_env, gid, rustc_span::DUMMY_SP)
                        .ok()?
                    {
                        // the constant evaluated to a scalar. note, this could still
                        // be a thin pointer!
                        rustc_middle::mir::ConstValue::Scalar(s) => Some(ConstSource::Scalar(s)),

                        // the constant evaluated to buffer, which can be indexed into
                        // to actually read values
                        rustc_middle::mir::ConstValue::Indirect { alloc_id, offset } => {
                            let alloc = tcx.global_alloc(alloc_id).unwrap_memory();
                            Some(ConstSource::Indirect(alloc, offset))
                        }

                        // the constant evaluated to a fat pointer.
                        rustc_middle::mir::ConstValue::Slice { alloc_id, meta } => {
                            let alloc = tcx.global_alloc(alloc_id).unwrap_memory();
                            Some(ConstSource::Slice(alloc, meta))
                        }

                        // ZSTs are left unsupported, these don't even exist at runtime
                        // and should therefore just be ignored...
                        rustc_middle::mir::ConstValue::ZeroSized => None,
                    }
                }

                rustc_hir::ItemKind::Static(rustc_ast::Mutability::Not, ..) => {
                    // run static evaluation. statics always evalute to a buffer + offset.
                    let alloc = tcx.eval_static_initializer(did).ok()?;
                    Some(ConstSource::Indirect(alloc, rustc_abi::Size::ZERO))
                }

                // the ldid is not referring to a constant... ?
                _ => None,
            },

            // the ldid is not referring to an item... ?
            _ => None,
        }
    }
}

/// A handle to the const-evaluated value of a global, threaded alongside the
/// recursive type-walk done in add_var so each leaf var-decl can be tagged with its
/// compile-time bytes.
#[derive(Clone, Copy)]
pub enum ConstSource<'tcx> {
    /// A primitive scalar value (e.g. i32, but also thin pointers)
    Scalar(rustc_middle::mir::interpret::Scalar),

    /// An allocation containing the value, plus a byte offset. Used for
    /// aggregate consts and non-mut statics
    Indirect(
        rustc_middle::mir::interpret::ConstAllocation<'tcx>,
        rustc_abi::Size,
    ),

    /// Fat-pointer-like: a slice body allocation (.0), with (.1) elements.
    Slice(rustc_middle::mir::interpret::ConstAllocation<'tcx>, u64),
}

impl<'tcx> ConstSource<'tcx> {
    /// Project into the i-th field of an aggregate of type `parent_ty`
    /// (struct, tuple, or array)
    pub fn project_field(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        parent_ty: &rustc_middle::ty::Ty<'tcx>,
        idx: usize,
    ) -> Option<Self> {
        let Self::Indirect(alloc, offset) = self else {
            return None;
        };

        // get type of parent struct
        let ty_env = rustc_middle::ty::TypingEnv::fully_monomorphized();
        let layout = tcx.layout_of(ty_env.as_query_input(*parent_ty)).ok()?;

        // determine how deep into the struct our field of interest lies
        let field_offset = layout.fields.offset(idx);

        // and offset into the alloc by the appropriate amount.
        Some(Self::Indirect(*alloc, *offset + field_offset))
    }

    /// Project into the i-th element of a Slice
    /// e.g. (Slice(&[1, 2, 3]) @ index 1 --> Indirect(2)).
    pub fn project_slice_elem(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        elem_ty: rustc_middle::ty::Ty<'tcx>,
        idx: u64,
    ) -> Option<Self> {
        let Self::Slice(alloc, _) = self else {
            return None;
        };

        // get the type of the element to determine how much to offset by
        let ty_env = rustc_middle::ty::TypingEnv::fully_monomorphized();
        let elem_layout = tcx.layout_of(ty_env.as_query_input(elem_ty)).ok()?;

        // and then actually offset into the allocation
        Some(Self::Indirect(*alloc, elem_layout.size * idx))
    }

    /// Returns a length of a slice, if this ConstSource is a slice.
    pub fn slice_len(&self) -> Option<u64> {
        match self {
            Self::Slice(_, len) => Some(*len),
            _ => None,
        }
    }

    /// If this source is a Scalar(Ptr), in other words, a thin pointer,
    /// pointing at a Sized memory alloc, follow the pointer and return an Indirect source onto the
    /// pointee bytes.
    ///
    /// Returns None for non-pointer scalars or pointers that don't point at a memory alloc.
    pub fn deref_ptr(&self, tcx: rustc_middle::ty::TyCtxt<'tcx>) -> Option<Self> {
        let Self::Scalar(rustc_middle::mir::interpret::Scalar::Ptr(ptr, _)) = self else {
            return None;
        };
        let (prov, offset) = ptr.into_raw_parts();
        let alloc_id = prov.alloc_id();
        match tcx.global_alloc(alloc_id) {
            rustc_middle::mir::interpret::GlobalAlloc::Memory(alloc) => {
                Some(Self::Indirect(alloc, offset))
            }
            _ => None,
        }
    }

    /// Read a fat pointer (data ptr + length) at the current Indirect
    /// location and return a Slice-like source pointing at the pointee bytes.
    /// This effectively extracts the const contents and the length, required
    /// to appropriately flatten out globals.
    pub fn load_fat_ptr_to_slice(&self, tcx: rustc_middle::ty::TyCtxt<'tcx>) -> Option<Self> {
        // fat pointer is necessarily an Indirect allocation type
        let Self::Indirect(alloc, offset) = self else {
            return None;
        };

        let inner = alloc.inner();
        let ptr_size = tcx.data_layout.pointer_size(); // nice!

        // first word in fat pointer allocation is going to be the data pointer
        // if it's not, then we are not looking at a well-formed slice

        // ptr_range is the sizeof(ptr) bytes that we are about to interpret
        // as the data pointer.
        let ptr_range = rustc_middle::mir::interpret::alloc_range(*offset, ptr_size);

        // read ptr_range bytes, ptr_scalar is now the actual ptr value.
        let ptr_scalar = inner.read_scalar(&tcx, ptr_range, true).ok()?;
        let rustc_middle::mir::interpret::Scalar::Ptr(ptr, _) = ptr_scalar else {
            return None;
        };

        // from the data pointer, extract the allocation buffer that holds the actual data.
        let (prov, _) = ptr.prov_and_relative_offset();
        let data_alloc = match tcx.global_alloc(prov.alloc_id()) {
            rustc_middle::mir::interpret::GlobalAlloc::Memory(a) => a,
            _ => return None,
        };

        // second word of fat pointer is the length of the slice.
        let len_range = rustc_middle::mir::interpret::alloc_range(*offset + ptr_size, ptr_size);
        let len_scalar = inner.read_scalar(&tcx, len_range, false).ok()?;
        let len = len_scalar.to_target_usize(&tcx).discard_err()?;

        // return a constructed slice type, with the buffer and length
        Some(Self::Slice(data_alloc, len))
    }

    /// Read the value at the current location as a primitive of type ty
    /// Returns None if ty isn't a primitive or the bytes can't be read as a scalar.
    pub fn read_leaf(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        ty: &rustc_middle::ty::Ty<'tcx>,
    ) -> Option<String> {
        let scalar = match self {
            Self::Scalar(s) => *s,
            Self::Indirect(alloc, offset) => {
                let ty_env = rustc_middle::ty::TypingEnv::fully_monomorphized();
                let layout = tcx.layout_of(ty_env.as_query_input(*ty)).ok()?;
                let range = rustc_middle::mir::interpret::alloc_range(*offset, layout.size);
                alloc.inner().read_scalar(&tcx, range, false).ok()?
            }
            Self::Slice(_, _) => return None,
        };
        Some(format_scalar(scalar, ty))
    }

    /// Decode a Slice source as a UTF-8 string, surrounded by quotes. Used
    /// for &str consts.
    pub fn read_str(&self) -> Option<String> {
        let Self::Slice(alloc, len) = self else {
            return None;
        };
        let inner = alloc.inner();
        let bytes = inner.inspect_with_uninit_and_ptr_outside_interpreter(0..(*len as usize));
        let s = std::str::from_utf8(bytes).ok()?;
        Some(format!(
            "\"{}\"",
            s.replace('\\', "\\\\").replace('"', "\\\"")
        ))
    }
}

fn format_scalar(
    scalar: rustc_middle::mir::interpret::Scalar,
    ty: &rustc_middle::ty::Ty<'_>,
) -> String {
    use rustc_type_ir::TyKind;
    match ty.kind() {
        TyKind::Bool => match scalar_bits(scalar) {
            Some(0) => "false".to_string(),
            Some(_) => "true".to_string(),
            None => scalar.to_string(),
        },
        TyKind::Char => scalar_bits(scalar)
            .and_then(|b| char::from_u32(b as u32))
            .map(|c| format!("'{}'", c))
            .unwrap_or_else(|| scalar.to_string()),
        _ => scalar.to_string(),
    }
}

fn scalar_bits(scalar: rustc_middle::mir::interpret::Scalar) -> Option<u128> {
    match scalar {
        rustc_middle::mir::interpret::Scalar::Int(i) => Some(i.to_bits(i.size())),
        rustc_middle::mir::interpret::Scalar::Ptr(_, _) => None,
    }
}
