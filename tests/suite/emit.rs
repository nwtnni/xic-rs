#[test_generator::test_resources("tests/execute/*.xi")]
pub fn interpret(path: &str) -> anyhow::Result<()> {
    let hir = super::emit_hir(path)?;
    let lir = hir.map_ref(xic::api::emit_lir);

    let hir_stdout = super::interpret_hir(&hir)?;
    let lir_stdout = super::interpret_lir(&lir)?;

    pretty_assertions::assert_eq!(hir_stdout, lir_stdout);
    insta::assert_snapshot!(path, lir_stdout);
    Ok(())
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn reorder(path: &str) -> anyhow::Result<()> {
    let expected_stdout = super::execute_expected(path)?;

    let reordered = super::reorder(path)?;
    let reordered_stdout = super::interpret_lir(&reordered)?;

    let cleaned = reordered
        .map(xic::api::construct_cfg)
        .map_mut(xic::api::clean_cfg)
        .map(xic::api::destruct_cfg);
    let cleaned_stdout = super::interpret_lir(&cleaned)?;

    pretty_assertions::assert_eq!(expected_stdout, reordered_stdout);
    pretty_assertions::assert_eq!(expected_stdout, cleaned_stdout);
    Ok(())
}
