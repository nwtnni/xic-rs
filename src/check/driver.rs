use std::io::{BufWriter, Write};

use crate::check;
use crate::data::ast;
use crate::error;
use crate::util::Tap;

pub struct Driver<'main> {
    lib: Option<&'main std::path::PathBuf>,
    directory: &'main std::path::Path,
    diagnostic: bool,
}

impl<'main> Driver<'main> {
    pub fn new(
        directory: &'main std::path::Path,
        diagnostic: bool,
        lib: Option<&'main std::path::PathBuf>,
    ) -> Self {
        Driver {
            lib,
            directory,
            diagnostic,
        }
    }

    pub fn drive(
        &self,
        path: &std::path::Path,
        ast: &ast::Program,
    ) -> Result<check::Env, error::Error> {
        let lib = if let Some(path) = self.lib {
            path
        } else {
            path.parent().unwrap()
        };
        let checker = check::Checker::new();
        let checked = checker.check_program(lib, ast);

        if self.diagnostic {
            let mut log = self
                .directory
                .join(path)
                .with_extension("typed")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            match &checked {
                Ok(_) => write!(&mut log, "Valid Xi Program")?,
                Err(error) => write!(&mut log, "{}", error)?,
            }
        }

        checked
    }
}
