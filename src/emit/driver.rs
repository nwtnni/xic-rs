use std::io::BufWriter;

use crate::check;
use crate::data::ast;
use crate::data::ir;
use crate::data::lir;
use crate::emit;
use crate::emit::Foldable;
use crate::interpret;
use crate::util::sexp::Serialize;
use crate::util::Tap;

#[derive(Debug)]
pub struct Driver<'main> {
    directory: &'main std::path::Path,
    diagnostic: bool,
    fold: bool,
    run: bool,
}

impl<'main> Driver<'main> {
    pub fn new(directory: &'main std::path::Path, diagnostic: bool, fold: bool, run: bool) -> Self {
        Driver {
            directory,
            diagnostic,
            fold,
            run,
        }
    }

    pub fn drive(
        &self,
        path: &std::path::Path,
        ast: &ast::Program,
        env: &check::Context,
    ) -> anyhow::Result<ir::Unit<lir::Function>> {
        let canonizer = emit::Canonizer::new();
        let emitter = emit::Emitter::new(env);
        let mut hir = emitter.emit_unit(path, ast);
        if self.fold {
            hir = hir.fold();
        }

        if self.diagnostic {
            let mut log = self
                .directory
                .join(path)
                .with_extension("hir")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            hir.sexp().write(80, &mut log)?;
        }

        if self.run {
            interpret::hir::interpret_unit(&hir)?;
        }

        let mut lir = canonizer.canonize_unit(hir);

        if self.fold {
            lir = lir.fold();
        }

        if self.diagnostic {
            let mut log = self
                .directory
                .join(path)
                .with_extension("lir")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            lir.sexp().write(80, &mut log)?;
        }

        if self.run {
            interpret::lir::interpret_unit(&lir)?;
        }

        Ok(lir)
    }
}
