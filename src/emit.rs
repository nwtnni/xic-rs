mod canonize;
#[allow(clippy::module_inception)]
mod emit;
mod print;

pub(crate) use canonize::Canonizer;

use std::path::Path;

use crate::check;
use crate::data::ast;
use crate::data::hir;
use crate::data::lir;

pub fn emit_hir(path: &Path, program: &ast::Program, context: &check::Context) -> hir::Unit {
    emit::emit_unit(path, context, program)
}

pub fn emit_lir(hir: &hir::Unit) -> lir::Unit<lir::Label> {
    Canonizer::new().canonize_unit(hir)
}
