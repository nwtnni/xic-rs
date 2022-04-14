mod checker;
mod context;
mod error;

pub(crate) use checker::Checker;
pub(crate) use context::Context;
pub(crate) use context::Entry;
pub(crate) use error::Error;
pub(crate) use error::ErrorKind;

use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;

use crate::data::ast;
use crate::util::Tap as _;

pub fn check(
    path: &Path,
    library: Option<&Path>,
    diagnostic: Option<&Path>,
    program: &ast::Program,
) -> Result<Context, crate::Error> {
    let library = if let Some(path) = library {
        path
    } else {
        path.parent().unwrap()
    };

    let checker = Checker::new();
    let checked = checker.check_program(library, program);

    if let Some(directory) = diagnostic {
        let mut log = directory
            .join(path)
            .with_extension("typed")
            .tap(fs::File::create)
            .map(io::BufWriter::new)?;

        match &checked {
            Ok(_) => write!(&mut log, "Valid Xi Program")?,
            Err(error) => write!(&mut log, "{}", error)?,
        }
    }

    checked
}
