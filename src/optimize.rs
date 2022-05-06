mod constant_fold;
mod dead_code;

pub use dead_code::eliminate as eliminate_dead_code;

use crate::data::hir;
use crate::data::lir;
use constant_fold::Foldable as _;

pub fn constant_fold_hir(hir: hir::Function) -> hir::Function {
    hir.fold()
}

pub fn constant_fold_lir<T: lir::Target>(lir: lir::Function<T>) -> lir::Function<T> {
    lir.fold()
}
