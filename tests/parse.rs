use std::fmt;
use std::path::Path;

use xic::data::sexp::Serialize as _;

struct Snapshot(Result<xic::data::ast::Program, xic::Error>);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Ok(program) => write!(fmt, "{}", program.sexp()),
            Err(error) => write!(fmt, "{}", error),
        }
    }
}

#[test_generator::test_resources("tests/parse/*.xi")]
pub fn parse(path: &str) {
    let lexed = xic::api::lex(Path::new(path)).unwrap();
    let parsed = xic::api::parse(lexed);

    insta::assert_display_snapshot!(path, Snapshot(parsed));
}
