use std::fmt;
use std::path::Path;

struct Snapshot(Vec<xic::lex::Spanned>);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for spanned in &self.0 {
            match spanned {
                Ok((left, token, _)) => writeln!(fmt, "{} {}", left, token)?,
                Err(error) => writeln!(fmt, "{}", error)?,
            }
        }
        Ok(())
    }
}

#[test_generator::test_resources("tests/lex/*.xi")]
pub fn lex(path: &str) {
    let lexer = xic::lex::Driver::new(Path::new("."), false);
    let lexed = lexer.drive(Path::new(path)).unwrap();
    insta::assert_display_snapshot!(path, Snapshot(lexed));
}
