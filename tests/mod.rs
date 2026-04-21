#![feature(rustc_private)]

#[test]
fn test() {
    let cwd = std::env::current_dir().unwrap();
    let test_file = cwd.join("tests/simple/main.rs");
    decls_gen::DeclsFile::from_source_file(&test_file);
}