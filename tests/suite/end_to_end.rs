use std::iter;
use std::process::Command;

fn compile(path: &str) -> String {
    Command::new(env!("CARGO_BIN_EXE_xic"))
        .arg("-d")
        .arg("-")
        .arg(path)
        .output()
        .map(|output| String::from_utf8(output.stdout))
        .unwrap()
        .unwrap()
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
}
