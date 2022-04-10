use std::io::BufWriter;

use crate::check;
use crate::data::ast;
use crate::data::ir;
use crate::data::lir;
use crate::emit;
use crate::emit::Foldable;
use crate::error;
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
    ) -> Result<ir::Unit<lir::Function>, error::Error> {
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
                .with_extension("ir")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            hir.sexp().write(80, &mut log)?;
        }

        if self.run {
            interpret::hir::Global::run(&hir);
        }

        let mut lir = canonizer.canonize_unit(hir);
        if self.fold {
            lir = lir.fold();
        }

        Ok(lir)
    }
}
