mod canonizer;
mod emitter;
mod printer;

pub(crate) use canonizer::Canonizer;
pub(crate) use emitter::Emitter;

use std::path::Path;

use crate::check;
use crate::data::ast;
use crate::data::hir;
use crate::data::ir;
use crate::data::lir;

pub fn emit_hir(
    path: &Path,
    program: &ast::Program,
    context: &check::Context,
) -> ir::Unit<hir::Function> {
    Emitter::new(context).emit_unit(path, program)
}

pub fn emit_lir(hir: &ir::Unit<hir::Function>) -> ir::Unit<lir::Function> {
    Canonizer::new().canonize_unit(hir)
}
