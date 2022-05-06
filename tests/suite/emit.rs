#[test_generator::test_resources("tests/execute/*.xi")]
pub fn interpret(path: &str) {
    let hir = super::emit_hir(path);
    let lir = hir.map_ref(xic::api::emit_lir);

    let hir_stdout = super::interpret_hir(&hir);
    let lir_stdout = super::interpret_lir(&lir);

    pretty_assertions::assert_eq!(hir_stdout, lir_stdout);
    insta::assert_snapshot!(path, lir_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn reorder(path: &str) {
    let lir = super::emit_lir(path);
    let lir_stdout = super::interpret_lir(&lir);

    let cfg = lir.map(xic::api::construct_cfg).map(xic::api::destruct_cfg);
    let cfg_stdout = super::interpret_lir(&cfg);

    pretty_assertions::assert_eq!(lir_stdout, cfg_stdout);
}
