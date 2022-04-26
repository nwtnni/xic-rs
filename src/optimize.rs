mod folder;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use folder::Foldable as _;

pub fn fold_hir(hir: ir::Unit<hir::Function>) -> ir::Unit<hir::Function> {
    hir.fold()
}

pub fn fold_lir<T: lir::Target>(lir: ir::Unit<lir::Function<T>>) -> ir::Unit<lir::Function<T>> {
    lir.fold()
}
