#[test_generator::test_resources("tests/execute/*.xi")]
pub fn tile(path: &str) -> anyhow::Result<()> {
    let expected_stdout = super::execute_expected(path)?;

    let assembly = super::tile(path)?.map_ref(xic::api::allocate_trivial);
    let assembly_stdout = super::execute(&assembly)?;

    pretty_assertions::assert_eq!(expected_stdout, assembly_stdout);
    Ok(())
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn reorder(path: &str) -> anyhow::Result<()> {
    let expected_stdout = super::execute_expected(path)?;

    let cfg = super::tile(path)?.map(xic::api::construct_cfg);

    let reordered = cfg
        .clone()
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);
    let reordered_stdout = super::execute(&reordered)?;

    pretty_assertions::assert_eq!(expected_stdout, reordered_stdout);

    let cleaned = cfg
        .map_mut(xic::api::clean_cfg)
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);
    let cleaned_stdout = super::execute(&cleaned)?;

    pretty_assertions::assert_eq!(expected_stdout, cleaned_stdout);
    Ok(())
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn allocate(path: &str) -> anyhow::Result<()> {
    let expected_stdout = super::execute_expected(path)?;

    let linear = super::tile(path)?
        .map(xic::api::construct_cfg)
        .map(xic::api::allocate_linear);

    let linear_stdout = super::execute(&linear)?;

    pretty_assertions::assert_eq!(expected_stdout, linear_stdout);
    Ok(())
}
