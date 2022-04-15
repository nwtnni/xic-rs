use std::fmt;
use std::path::Path;

use xic::data::ast;
use xic::data::sexp::Serialize as _;

struct Snapshot(Result<ast::Program, xic::Error>);

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
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    let program = xic::api::parse(tokens);

    insta::assert_display_snapshot!(path, Snapshot(program));
}
