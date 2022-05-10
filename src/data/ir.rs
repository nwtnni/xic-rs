use std::collections::BTreeMap;

use crate::data::ast;
use crate::data::operand::Label;
use crate::data::symbol::Symbol;

#[derive(Clone, Debug)]
pub struct Unit<T> {
    pub name: Symbol,
    pub functions: BTreeMap<Symbol, T>,
    pub data: BTreeMap<Symbol, Label>,
}

impl<T> Unit<T> {
    pub fn map<F: FnMut(T) -> U, U>(self, mut apply: F) -> Unit<U> {
        Unit {
            name: self.name,
            functions: self
                .functions
                .into_iter()
                .map(|(symbol, function)| (symbol, apply(function)))
                .collect(),
            data: self.data,
        }
    }

    pub fn map_ref<'a, F: FnMut(&'a T) -> U, U>(&'a self, mut apply: F) -> Unit<U> {
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

    pub fn map_mut<F: FnMut(&mut T)>(mut self, mut apply: F) -> Self {
        self.functions
            .iter_mut()
            .for_each(|(_, function)| apply(function));
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// Unsigned above or equal (http://www.unixwiz.net/techtips/x86-jumps.html)
    ///
    /// Used for optimizing (0 <= signed < max) into (unsigned < max),
    /// relying on integer underflow.
    Ae,
}

impl Condition {
    pub fn negate(&self) -> Self {
        match self {
            Condition::Lt => Condition::Ge,
            Condition::Le => Condition::Gt,
            Condition::Ge => Condition::Lt,
            Condition::Gt => Condition::Ge,
            Condition::Ne => Condition::Eq,
            Condition::Eq => Condition::Ne,
            Condition::Ae => unreachable!(),
        }
    }
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
