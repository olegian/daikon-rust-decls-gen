#![feature(rustc_private)]

#[test]
fn test() {
    let cwd = std::env::current_dir().unwrap();
    let test_file = cwd.join("tests/simple/main.rs");
    let decls = decls_gen::DeclsFile::from_source_file(&test_file);

    let out_file = cwd.join("tests/simple/simple.decls");
    decls.write_to_file(&out_file);
    // println!("{:#?}", decls);
}