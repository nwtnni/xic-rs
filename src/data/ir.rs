use std::collections::HashMap;

use crate::data::ast;
use crate::data::hir;
use crate::data::lir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Unit<F: IR> {
    pub name: symbol::Symbol,
    pub functions: HashMap<symbol::Symbol, F>,
    pub data: HashMap<symbol::Symbol, operand::Label>,
}

pub trait IR {}
impl IR for hir::Function {}
impl IR for lir::Function {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Binary {
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

impl From<ast::Binary> for Binary {
    fn from(binary: ast::Binary) -> Self {
        match binary {
            ast::Binary::Mul => Binary::Mul,
            ast::Binary::Hul => Binary::Hul,
            ast::Binary::Div => Binary::Div,
            ast::Binary::Mod => Binary::Mod,
            ast::Binary::Add => Binary::Add,
            ast::Binary::Sub => Binary::Sub,
            ast::Binary::Lt => Binary::Lt,
            ast::Binary::Le => Binary::Le,
            ast::Binary::Ge => Binary::Ge,
            ast::Binary::Gt => Binary::Gt,
            ast::Binary::Eq => Binary::Eq,
            ast::Binary::Ne => Binary::Ne,
            ast::Binary::And => Binary::And,
            ast::Binary::Or => Binary::Or,
        }
    }
}
