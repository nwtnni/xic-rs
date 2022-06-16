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

#[test_generator::test_resources("tests/advent/advent*.xi")]
pub fn advent(path: &str) -> anyhow::Result<()> {
    let string = compile("tests/advent/string.xi")?;
    let advent = compile(path)?;
    let stdout = super::execute_all([string, advent])?;
    insta::assert_snapshot!(path, stdout);
    Ok(())
}

mod separate {
    macro_rules! test {
        ($name:ident, $($file:ident),* $(,)?) => {
            #[test]
            fn $name() -> anyhow::Result<()> {
                $(
                    let $file = super::compile(concat!("tests/separate/", stringify!($file), ".xi"))?;
                )*
                let stdout = super::super::execute_all([$($file),*])?;
                insta::assert_display_snapshot!(stdout);
                Ok(())
            }
        }
    }

    test!(smoke, smoke_1, smoke_2);
    test!(cycle_function, cycle_function_1, cycle_function_2);
    test!(out_of_order, out_of_order_1, out_of_order_2);
    test!(generic_class, generic_class_1, generic_class_2);
    test!(generic_function, generic_function_1, generic_function_2);
}
