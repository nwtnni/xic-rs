#[test_generator::test_resources("tests/execute/*.xi")]
pub fn constant_fold_hir(path: &str) {
    let hir = super::emit_hir(path);

    let hir_stdout = super::interpret_hir(&hir);
    let hir_folded = hir.map(xic::api::optimize::constant_fold);
    let hir_folded_stdout = super::interpret_hir(&hir_folded);

    pretty_assertions::assert_eq!(hir_stdout, hir_folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn constant_fold_lir(path: &str) {
    let lir = super::emit_lir(path);

    let lir_stdout = super::interpret_lir(&lir);
    let lir_folded = lir.map(xic::api::optimize::constant_fold);
    let lir_folded_stdout = super::interpret_lir(&lir_folded);

    pretty_assertions::assert_eq!(lir_stdout, lir_folded_stdout);
}
