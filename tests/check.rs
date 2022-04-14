use std::fmt;
use std::path::Path;

struct Snapshot(Result<(), xic::Error>);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Ok(()) => write!(fmt, "Valid Xi Program"),
            Err(error) => write!(fmt, "{}", error),
        }
    }
}

#[test_generator::test_resources("tests/check/*.xi")]
pub fn check(path: &str) {
    let lexed = xic::api::lex(Path::new(path), None).unwrap();
    let parsed = xic::api::parse(Path::new(path), None, lexed).unwrap();
    let checked = xic::api::check(Path::new(path), None, None, &parsed).map(|_| ());

    insta::assert_display_snapshot!(path, Snapshot(checked));
}
