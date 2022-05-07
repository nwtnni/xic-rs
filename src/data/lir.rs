use std::fmt;

use crate::abi;
use crate::data::ir;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::sexp;
use crate::data::sexp::Serialize;
use crate::data::symbol::Symbol;

pub const ZERO: Expression = Expression::Immediate(Immediate::Integer(0));
pub const ONE: Expression = Expression::Immediate(Immediate::Integer(1));
pub const EIGHT: Expression = Expression::Immediate(Immediate::Integer(abi::WORD));

pub type Unit<T> = ir::Unit<Function<T>>;

impl<T: Target> fmt::Display for Unit<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

pub struct Function<T: Target> {
    pub name: Symbol,
    pub statements: Vec<Statement<T>>,
    pub arguments: usize,
    pub returns: usize,
    pub enter: T::Access,
    pub exit: T::Access,
}

impl<T: Target> Function<T> {
    pub fn callee_arguments(&self) -> usize {
        self.statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::Call(_, arguments, _) => Some(arguments.len()),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }
}

impl<T: Target> fmt::Display for Function<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Argument(usize),
    Return(usize),
    Immediate(Immediate),
    Temporary(Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

impl From<i64> for Expression {
    fn from(integer: i64) -> Self {
        Expression::Immediate(Immediate::Integer(integer))
    }
}

impl From<operand::Label> for Expression {
    fn from(label: operand::Label) -> Self {
        Expression::Immediate(Immediate::Label(label))
    }
}

impl From<Temporary> for Expression {
    fn from(temporary: Temporary) -> Self {
        Expression::Temporary(temporary)
    }
}

impl From<Register> for Expression {
    fn from(register: Register) -> Self {
        Expression::Temporary(Temporary::Register(register))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement<T> {
    Jump(operand::Label),
    CJump {
        condition: ir::Condition,
        left: Expression,
        right: Expression,
        r#true: operand::Label,
        r#false: T,
    },
    Call(Expression, Vec<Expression>, usize),
    Label(operand::Label),
    Move {
        destination: Expression,
        source: Expression,
    },
    Return(Vec<Expression>),
}

impl<T: Serialize> fmt::Display for Statement<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fallthrough;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Label(pub(crate) operand::Label);

pub trait Target: sexp::Serialize + Copy + Clone + Eq + fmt::Debug {
    type Access;
    fn target(&self) -> Option<&operand::Label>;
    fn target_mut(&mut self) -> Option<&mut operand::Label>;
    fn access(access: &Self::Access) -> Option<&operand::Label>;
}

impl Target for Fallthrough {
    type Access = operand::Label;

    fn target(&self) -> Option<&operand::Label> {
        None
    }

    fn target_mut(&mut self) -> Option<&mut operand::Label> {
        None
    }

    fn access(access: &Self::Access) -> Option<&operand::Label> {
        Some(access)
    }
}

impl Target for Label {
    type Access = ();

    fn target(&self) -> Option<&operand::Label> {
        Some(&self.0)
    }

    fn target_mut(&mut self) -> Option<&mut operand::Label> {
        Some(&mut self.0)
    }

    fn access((): &Self::Access) -> Option<&operand::Label> {
        None
    }
}
