#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;
extern crate rustc_const_eval;
extern crate rustc_ast_ir;
extern crate rustc_abi;

mod callbacks;
mod fields;
mod ppt;

pub mod decls;
pub use decls::DeclsFile;
