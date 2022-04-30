use std::io;
use std::path::Path;

use xic::data::hir;
use xic::data::ir;

fn compile(path: &str) -> ir::Unit<hir::Function> {
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    let program = xic::api::parse(tokens).unwrap();
    let context = xic::api::check(Path::new(path).parent().unwrap(), &program).unwrap();
    xic::api::emit_hir(Path::new(path), &program, &context)
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn interpret(path: &str) {
    let hir = compile(path);
    let lir = xic::api::emit_lir(&hir);

    let mut hir_stdin = io::Cursor::new(Vec::new());
    let mut hir_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_hir(&hir, &mut hir_stdin, &mut hir_stdout).unwrap();

    let mut lir_stdin = io::Cursor::new(Vec::new());
    let mut lir_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_lir(&lir, &mut lir_stdin, &mut lir_stdout).unwrap();

    let hir_stdout = String::from_utf8(hir_stdout.into_inner()).unwrap();
    let lir_stdout = String::from_utf8(lir_stdout.into_inner()).unwrap();

    pretty_assertions::assert_eq!(hir_stdout, lir_stdout);
    insta::assert_snapshot!(path, lir_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn reorder(path: &str) {
    let hir = compile(path);
    let lir = xic::api::emit_lir(&hir);

    let mut lir_stdin = io::Cursor::new(Vec::new());
    let mut lir_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_lir(&lir, &mut lir_stdin, &mut lir_stdout).unwrap();

    let cfg = xic::api::construct_control_flow(&lir);
    let cfg = xic::api::destruct_control_flow(&cfg);

    let mut cfg_stdin = io::Cursor::new(Vec::new());
    let mut cfg_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_lir(&cfg, &mut cfg_stdin, &mut cfg_stdout).unwrap();

    let lir_stdout = String::from_utf8(lir_stdout.into_inner()).unwrap();
    let cfg_stdout = String::from_utf8(cfg_stdout.into_inner()).unwrap();

    pretty_assertions::assert_eq!(lir_stdout, cfg_stdout);
}
