mod constant_fold;

use crate::data::hir;
use crate::data::lir;
use constant_fold::Foldable as _;

pub fn constant_fold_hir(hir: hir::Unit) -> hir::Unit {
    hir.fold()
}

pub fn constant_fold_lir<T: lir::Target>(lir: lir::Unit<T>) -> lir::Unit<T> {
    lir.fold()
}
