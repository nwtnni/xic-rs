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
    let parser = xic::parse::Driver::new(Path::new("."), false);
    let checker = xic::check::Driver::new(Path::new("."), false, None);

    let lexed = xic::lex(Path::new(path), None).unwrap();
    let parsed = parser.drive(Path::new(path), lexed).unwrap();
    let checked = checker.drive(Path::new(path), &parsed).map(|_| ());

    insta::assert_display_snapshot!(path, Snapshot(checked));
}
