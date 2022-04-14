use std::fmt;
use std::path::Path;

struct Snapshot<P, T, E>(Vec<Result<(P, T, P), E>>);

impl<P: fmt::Display, T: fmt::Display, E: fmt::Display> fmt::Display for Snapshot<P, T, E> {
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
    let lexed = xic::api::lex(Path::new(path), None).unwrap();
    insta::assert_display_snapshot!(path, Snapshot(lexed));
}
