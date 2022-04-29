use std::collections::BTreeMap;

use crate::data::ast;
use crate::data::operand;
use crate::data::symbol;

#[derive(Clone, Debug)]
pub struct Unit<T> {
    pub name: symbol::Symbol,
    pub functions: BTreeMap<symbol::Symbol, T>,
    pub data: BTreeMap<symbol::Symbol, operand::Label>,
}

#[derive(Clone, Debug)]
pub struct Function<T> {
    pub name: symbol::Symbol,
    pub statements: T,
    pub arguments: usize,
    pub returns: usize,
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
            ast::Binary::And => Binary::And,
            ast::Binary::Or => Binary::Or,
            binary => panic!(
                "[INTERNAL ERROR]: converting {:?} to IR binary operator",
                binary
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    Lt,
    Le,
    Ge,
    Gt,
    Ne,
    Eq,
}

impl From<ast::Binary> for Condition {
    fn from(binary: ast::Binary) -> Self {
        match binary {
            ast::Binary::Lt => Condition::Lt,
            ast::Binary::Le => Condition::Le,
            ast::Binary::Ge => Condition::Ge,
            ast::Binary::Gt => Condition::Gt,
            ast::Binary::Ne => Condition::Ne,
            ast::Binary::Eq => Condition::Eq,
            binary => panic!("[INTERNAL ERROR]: converting {:?} to IR condition", binary),
        }
    }
}
