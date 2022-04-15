use std::fmt;
use std::path::Path;

use xic::data::hir;
use xic::data::ir;
use xic::data::sexp::Serialize as _;

struct Snapshot(ir::Unit<hir::Function>);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0.sexp())
    }
}

#[test_generator::test_resources("tests/emit-hir/*.xi")]
pub fn emit_hir(path: &str) {
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    let program = xic::api::parse(tokens).unwrap();
    let context = xic::api::check(Path::new("tests/emit"), &program).unwrap();
    let hir = xic::api::emit_hir(Path::new(path), &program, &context);
    insta::assert_display_snapshot!(path, Snapshot(hir));
}
