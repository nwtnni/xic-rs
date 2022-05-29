use std::io::Read as _;
use std::iter;
use std::process::Command;
use std::process::Stdio;

fn compile(path: &str) -> String {
    let mut xic = Command::new(env!("CARGO_BIN_EXE_xic"))
        .stdout(Stdio::piped())
        .arg("-d")
        .arg("-")
        .arg(path)
        .spawn()
        .unwrap();

    let mut stdout = xic.stdout.take().unwrap();
    let mut buffer = Vec::new();
    let success = xic.wait().unwrap().success();

    if !success {
        panic!("`xic` invocation failed");
    }

    stdout.read_to_end(&mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn end_to_end(path: &str) {
    let expected_stdout = super::execute_expected(path);
    let stdout = super::execute(iter::once(compile(path)));
    pretty_assertions::assert_eq!(expected_stdout, stdout);
}

mod separate {
    #[test]
    fn smoke() {
        let smoke_1 = super::compile("tests/separate/smoke_1.xi");
        let smoke_2 = super::compile("tests/separate/smoke_2.xi");
        let stdout = super::super::execute([smoke_1, smoke_2]);
        insta::assert_display_snapshot!(stdout);
    }

    #[test]
    fn cycle_function() {
        let cycle_function_1 = super::compile("tests/separate/cycle_function_1.xi");
        let cycle_function_2 = super::compile("tests/separate/cycle_function_2.xi");
        let stdout = super::super::execute([cycle_function_1, cycle_function_2]);
        insta::assert_display_snapshot!(stdout);
    }
}
