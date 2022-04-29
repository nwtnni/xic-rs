mod fold;

use crate::data::hir;
use crate::data::lir;
use fold::Foldable as _;

pub fn fold_hir(hir: hir::Unit) -> hir::Unit {
    hir.fold()
}

pub fn fold_lir<T: lir::Target>(lir: lir::Unit<T>) -> lir::Unit<T> {
    lir.fold()
}
