#[test_generator::test_resources("tests/lex/*.xi")]
pub fn lex(path: &str) -> anyhow::Result<()> {
    let tokens = super::lex(path)?;
    insta::assert_display_snapshot!(path, tokens);
    Ok(())
}
