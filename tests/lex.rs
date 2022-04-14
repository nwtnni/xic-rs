use std::fmt;
use std::path::Path;

struct Snapshot(xic::data::token::Tokens);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.0)
    }
}

#[test_generator::test_resources("tests/lex/*.xi")]
pub fn lex(path: &str) {
    let lexed = xic::api::lex(Path::new(path), None).unwrap();
    insta::assert_display_snapshot!(path, Snapshot(lexed));
}
