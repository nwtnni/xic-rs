use std::iter;
use std::process::Command;

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn end_to_end(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let xic = Command::new(env!("CARGO_BIN_EXE_xic"))
        .arg("-d")
        .arg("-")
        .arg("--libpath")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/lib"))
        .arg(path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(xic.stdout)
        .map(iter::once)
        .map(super::execute)
        .unwrap();

    pretty_assertions::assert_eq!(expected_stdout, stdout);
}
