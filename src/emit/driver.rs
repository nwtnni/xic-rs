use std::io;
use std::io::BufWriter;

use crate::check;
use crate::data::ast;
use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::emit;
use crate::emit::Foldable;
use crate::interpret;
use crate::data::sexp::Serialize;
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

    pub fn emit_hir(
        &self,
        path: &std::path::Path,
        ast: &ast::Program,
        env: &check::Context,
    ) -> anyhow::Result<ir::Unit<hir::Function>> {
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

        Ok(hir)
    }

    pub fn emit_lir(
        &self,
        path: &std::path::Path,
        hir: &ir::Unit<hir::Function>,
    ) -> anyhow::Result<ir::Unit<lir::Function>> {
        let canonizer = emit::Canonizer::new();
        let mut lir = canonizer.canonize_unit(hir.clone());

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
            interpret::lir::interpret_unit(&lir, io::BufReader::new(io::stdin()), io::stdout())?;
        }

        Ok(lir)
    }
}
