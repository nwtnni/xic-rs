use std::fmt;
use std::path::Path;

use xic::util::sexp::Serialize as _;

struct Snapshot(Result<xic::data::ast::Program, xic::Error>);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Ok(program) => {
                let mut buffer = Vec::new();
                program.sexp().write(80, &mut buffer).unwrap();
                write!(fmt, "{}", std::str::from_utf8(&buffer).unwrap())
            }
            Err(error) => write!(fmt, "{}", error),
        }
    }
}

#[test_generator::test_resources("tests/parse/*.xi")]
pub fn parse(path: &str) {
    let lexer = xic::lex::Driver::new(Path::new("."), false);
    let parser = xic::parse::Driver::new(Path::new("."), false);

    let lexed = lexer.drive(Path::new(path)).unwrap();
    let parsed = parser.drive(Path::new(path), lexed);

    insta::assert_display_snapshot!(path, Snapshot(parsed));
}
