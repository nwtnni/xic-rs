use std::fmt;

use crate::data::ir;
use crate::data::operand;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol::Symbol;

pub type Unit<T> = ir::Unit<Function<T>>;

impl<T: fmt::Display> Unit<T> {
    pub fn intel(&self) -> impl fmt::Display + '_ {
        crate::assemble::Intel(self)
    }
}

#[derive(Clone, Debug)]
pub struct Function<T> {
    pub name: Symbol,
    pub instructions: Vec<Assembly<T>>,
    pub arguments: usize,
    pub returns: usize,
    pub callee_arguments: usize,
    pub callee_returns: usize,
    pub caller_returns: Option<Temporary>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Assembly<T> {
    Binary(Binary, operand::Binary<T>),
    Unary(Unary, operand::Unary<T>),
    Nullary(Nullary),
    Label(Label),
    Jmp(Label),
    Jcc(Condition, Label),
}

impl<T: fmt::Display> fmt::Display for Assembly<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", crate::assemble::Intel(self))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Directive {
    Intel,
    Align(usize),
    Local(Label),
    Global(Label),
    Quad(Vec<i64>),
    Data,
    Text,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Binary {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Cmp,
    Mov,
    Lea,
}

impl From<ir::Binary> for Binary {
    fn from(binary: ir::Binary) -> Self {
        match binary {
            ir::Binary::Add => Binary::Add,
            ir::Binary::Sub => Binary::Sub,
            ir::Binary::Xor => Binary::Xor,
            ir::Binary::And => Binary::And,
            ir::Binary::Or => Binary::Or,
            _ => panic!("[INTERNAL ERROR]: converting unsupported IR operator"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unary {
    Neg,
    Call { arguments: usize, returns: usize },
    Mul,
    Hul,
    Div,
    Mod,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nullary {
    Cqo,
    Ret(usize, Option<Temporary>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    L,
    Le,
    Ge,
    G,
    Ne,
    E,
}

impl From<ir::Condition> for Condition {
    fn from(condition: ir::Condition) -> Self {
        match condition {
            ir::Condition::Lt => Condition::L,
            ir::Condition::Le => Condition::Le,
            ir::Condition::Ge => Condition::Ge,
            ir::Condition::Gt => Condition::G,
            ir::Condition::Ne => Condition::Ne,
            ir::Condition::Eq => Condition::E,
        }
    }
}
