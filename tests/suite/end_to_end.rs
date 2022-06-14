use std::process::Command;

use anyhow::Context as _;

fn compile(path: &str) -> anyhow::Result<String> {
    let mut xic = Command::new(env!("CARGO_BIN_EXE_xic"));

    xic.arg("-d").arg("-").arg(path);

    super::stdout(xic, None::<String>).context("Compiling with `xic`")
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn end_to_end(path: &str) -> anyhow::Result<()> {
    let expected_stdout = super::execute_expected(path)?;
    let stdout = super::execute(compile(path)?)?;
    pretty_assertions::assert_eq!(expected_stdout, stdout);
    Ok(())
}

#[test_generator::test_resources("tests/advent/*.xi")]
pub fn advent(path: &str) -> anyhow::Result<()> {
    let stdout = super::execute(compile(path)?)?;
    insta::assert_snapshot!(path, stdout);
    Ok(())
}

mod separate {
    #[test]
    fn smoke() -> anyhow::Result<()> {
        let smoke_1 = super::compile("tests/separate/smoke_1.xi")?;
        let smoke_2 = super::compile("tests/separate/smoke_2.xi")?;
        let stdout = super::super::execute_all([smoke_1, smoke_2])?;
        insta::assert_display_snapshot!(stdout);
        Ok(())
    }

    #[test]
    fn cycle_function() -> anyhow::Result<()> {
        let cycle_function_1 = super::compile("tests/separate/cycle_function_1.xi")?;
        let cycle_function_2 = super::compile("tests/separate/cycle_function_2.xi")?;
        let stdout = super::super::execute_all([cycle_function_1, cycle_function_2])?;
        insta::assert_display_snapshot!(stdout);
        Ok(())
    }

    #[test]
    fn out_of_order() -> anyhow::Result<()> {
        let out_of_order_1 = super::compile("tests/separate/out_of_order_1.xi")?;
        let out_of_order_2 = super::compile("tests/separate/out_of_order_2.xi")?;
        let stdout = super::super::execute_all([out_of_order_1, out_of_order_2])?;
        insta::assert_display_snapshot!(stdout);
        Ok(())
    }
}
