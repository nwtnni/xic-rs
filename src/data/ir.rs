use std::collections::BTreeMap;

use crate::data::ast;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Unit<T> {
    pub name: symbol::Symbol,
    pub functions: BTreeMap<symbol::Symbol, T>,
    pub data: BTreeMap<symbol::Symbol, operand::Label>,
}

impl<T> Unit<T> {
    pub fn map<'a, F: FnMut(&'a T) -> U, U>(&'a self, mut apply: F) -> Unit<U> {
        Unit {
            name: self.name,
            functions: self
                .functions
                .iter()
                .map(|(symbol, function)| (*symbol, apply(function)))
                .collect(),
            data: self.data.clone(),
        }
    }
}

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
            ast::Binary::Cat => unreachable!(),
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
