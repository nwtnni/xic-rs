mod canonize;
#[allow(clippy::module_inception)]
mod emit;
mod print;

use std::path::Path;

use crate::check;
use crate::data::ast;
use crate::data::hir;
use crate::data::lir;
use crate::data::operand::Label;

pub fn emit_hir(path: &Path, program: &ast::Program, context: &mut check::Context) -> hir::Unit {
    emit::emit_unit(path, context, program)
}

pub fn emit_lir(hir: &hir::Function) -> lir::Function<Label> {
    canonize::canonize_function(hir)
}
