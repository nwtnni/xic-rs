use std::collections::HashMap;

use crate::data::hir;
use crate::data::lir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Unit<F: IR> {
    pub name: symbol::Symbol,
    pub funs: HashMap<symbol::Symbol, F>,
    pub data: HashMap<symbol::Symbol, operand::Label>,
}

pub trait IR {}
impl IR for hir::Fun {}
impl IR for lir::Fun {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bin {
    Add,
    Sub,
    Mul,
    Hul,
    Div,
    Mod,
    Xor,
    Ls,
    Rs,
    ARs,
    Lt,
    Le,
    Ge,
    Gt,
    Ne,
    Eq,
    And,
    Or,
}
