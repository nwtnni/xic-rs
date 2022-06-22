use std::fmt;
use std::path::Path;

struct Snapshot<T>(Result<T, xic::Error>);

impl<T> fmt::Display for Snapshot<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Ok(_) => write!(fmt, "Valid Xi Program"),
            Err(error) => match error.report() {
                None => write!(fmt, "{}", error),
                Some(report) => {
                    let cache = xic::data::span::FileCache::default();
                    let mut buffer = Vec::new();

                    report
                        .with_config(ariadne::Config::default().with_color(false))
                        .finish()
                        .write(cache, &mut buffer)
                        .unwrap();

                    write!(fmt, "{}", String::from_utf8(buffer).unwrap())
                }
            },
        }
    }
}

#[test_generator::test_resources("tests/check/*.xi")]
pub fn check(path: &str) -> anyhow::Result<()> {
    let program = super::parse(path)?;
    let checked = xic::api::check(None, Path::new(path), program);

    insta::assert_display_snapshot!(path, Snapshot(checked));
    Ok(())
}
