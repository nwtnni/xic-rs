mod folder;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use folder::Foldable as _;

pub fn fold_hir(hir: ir::Unit<hir::Function>) -> ir::Unit<hir::Function> {
    hir.fold()
}

pub fn fold_lir(lir: ir::Unit<lir::Function>) -> ir::Unit<lir::Function> {
    lir.fold()
}
