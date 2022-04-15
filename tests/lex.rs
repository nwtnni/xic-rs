use std::fmt;
use std::path::Path;

use xic::data::token;

struct Snapshot(token::Tokens);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.0)
    }
}

#[test_generator::test_resources("tests/lex/*.xi")]
pub fn lex(path: &str) {
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    insta::assert_display_snapshot!(path, Snapshot(tokens));
}
