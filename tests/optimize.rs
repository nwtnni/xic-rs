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
pub fn fold_hir(path: &str) {
    let hir = compile(path);

    let mut hir_stdin = io::Cursor::new(Vec::new());
    let mut hir_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_hir(&hir, &mut hir_stdin, &mut hir_stdout).unwrap();

    let hir_folded = xic::api::fold_hir(hir);

    let mut hir_folded_stdin = io::Cursor::new(Vec::new());
    let mut hir_folded_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_hir(&hir_folded, &mut hir_folded_stdin, &mut hir_folded_stdout).unwrap();

    let hir_stdout = String::from_utf8(hir_stdout.into_inner()).unwrap();
    let hir_folded_stdout = String::from_utf8(hir_folded_stdout.into_inner()).unwrap();

    pretty_assertions::assert_eq!(hir_stdout, hir_folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn fold_lir(path: &str) {
    let hir = compile(path);
    let lir = hir.map(xic::api::emit_lir);

    let mut lir_stdin = io::Cursor::new(Vec::new());
    let mut lir_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_lir(&lir, &mut lir_stdin, &mut lir_stdout).unwrap();

    let lir_folded = xic::api::fold_lir(lir);

    let mut lir_folded_stdin = io::Cursor::new(Vec::new());
    let mut lir_folded_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_lir(&lir_folded, &mut lir_folded_stdin, &mut lir_folded_stdout).unwrap();

    let lir_stdout = String::from_utf8(lir_stdout.into_inner()).unwrap();
    let lir_folded_stdout = String::from_utf8(lir_folded_stdout.into_inner()).unwrap();

    pretty_assertions::assert_eq!(lir_stdout, lir_folded_stdout);
}
