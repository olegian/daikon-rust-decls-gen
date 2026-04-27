#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_ast_ir;
extern crate rustc_const_eval;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;

mod callbacks;
mod fields;
mod globals;
mod ppt;

pub mod decls;
pub mod vars;
pub use decls::DeclsFile;
pub use vars::VarName;
