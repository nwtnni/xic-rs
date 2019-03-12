use std::io::{Write, BufWriter};

use crate::check;
use crate::emit;
use crate::error;
use crate::data::ast;
use crate::util::Tap;
use crate::util::sexp::Serialize;

#[derive(Debug)]
pub struct Driver<'main> {
    directory: &'main std::path::Path,
    diagnostic: bool,
    fold: bool,
}

impl<'main> Driver<'main> {
    pub fn new(directory: &'main std::path::Path, diagnostic: bool, fold: bool) -> Self {
        Driver { directory, diagnostic, fold }
    }

    pub fn drive(&self, path: &std::path::Path, ast: &ast::Program, env: &check::Env) -> Result<(), error::Error> {
        let emitter = emit::Emitter::new(env);
        let hir = emitter.emit_program(ast);

        if self.diagnostic {
            let mut log = self.directory
                .join(path)
                .with_extension("ir")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            hir.sexp().write(80, &mut log)?;
        }

        Ok(())
    }
}
