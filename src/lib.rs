#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_ast;
extern crate rustc_middle;
extern crate rustc_hir;
extern crate rustc_span;

mod callbacks;

pub mod decls;
pub use decls::DeclsFile;
