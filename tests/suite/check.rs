use std::fmt;
use std::path::Path;

struct Snapshot<T>(Result<T, xic::Error>);

impl<T> fmt::Display for Snapshot<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Ok(_) => write!(fmt, "Valid Xi Program"),
            Err(error) => write!(fmt, "{}", error),
        }
    }
}

#[test_generator::test_resources("tests/check/*.xi")]
pub fn check(path: &str) {
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    let program = xic::api::parse(tokens).unwrap();
    let context = xic::api::check(Path::new("tests/check"), &program);

    insta::assert_display_snapshot!(path, Snapshot(context));
}
