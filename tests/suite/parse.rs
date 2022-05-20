use std::fmt;
use std::path::Path;

use xic::data::ast;

struct Snapshot(Result<ast::Program, xic::Error>);

impl fmt::Display for Snapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Ok(program) => write!(fmt, "{}", program),
            Err(error) => match error.report() {
                None => write!(fmt, "{}", error),
                Some(report) => {
                    let cache = xic::data::span::FileCache::default();
                    let mut buffer = Vec::new();

                    report
                        .with_config(ariadne::Config::default().with_color(false))
                        .finish()
                        .write(cache, &mut buffer)
                        .map_err(|_| fmt::Error)?;

                    write!(fmt, "{}", String::from_utf8(buffer).unwrap())
                }
            },
        }
    }
}

#[test_generator::test_resources("tests/parse/*.xi")]
pub fn parse(path: &str) {
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    let program = xic::api::parse(tokens);

    insta::assert_display_snapshot!(path, Snapshot(program));
}
