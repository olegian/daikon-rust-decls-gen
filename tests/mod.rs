#![feature(rustc_private)]

#[test]
fn simple() {
    let cwd = std::env::current_dir().unwrap();
    let test_file = cwd.join("tests/simple/main.rs");
    let decls = decls_gen::DeclsFile::from_source_file(&test_file);

    let out_file = cwd.join("tests/simple/simple.decls");
    decls.write_to_file(&out_file).unwrap();
}

#[test]
fn simple_parse() {
    let cwd = std::env::current_dir().unwrap();
    let test_file = cwd.join("tests/simple/main.rs");
    let decls = decls_gen::DeclsFile::from_source_file(&test_file);

    let out_file = cwd.join("tests/simple/simple.decls");
    decls.write_to_file(&out_file).unwrap();

    let decls_parsed = decls_gen::DeclsFile::from_decls_file(&out_file).unwrap();
    println!("{:#?}", decls_parsed);
}

#[test]
fn return_exits() {
    let cwd = std::env::current_dir().unwrap();
    let test_file = cwd.join("tests/return_exits/main.rs");
    let decls = decls_gen::DeclsFile::from_source_file(&test_file);

    let out_file = cwd.join("tests/return_exits/return_exits.decls");
    decls.write_to_file(&out_file).unwrap();
}