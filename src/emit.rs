mod canonize;
#[allow(clippy::module_inception)]
mod emit;
mod print;

pub(crate) use canonize::Canonizer;
pub(crate) use emit::Emitter;

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

pub fn emit_lir(hir: &ir::Unit<hir::Function>) -> ir::Unit<lir::Function<lir::Label>> {
    Canonizer::new().canonize_unit(hir)
}
