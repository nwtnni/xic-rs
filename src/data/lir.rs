use crate::constants;
use crate::data::ir;
use crate::data::operand;
use crate::data::symbol;

pub const ZERO: Expression = Expression::Immediate(operand::Immediate::Integer(0));
pub const ONE: Expression = Expression::Immediate(operand::Immediate::Integer(1));
pub const EIGHT: Expression =
    Expression::Immediate(operand::Immediate::Integer(constants::WORD_SIZE));

#[derive(Clone, Debug)]
pub struct Function<T> {
    pub name: symbol::Symbol,
    pub statements: Vec<Statement<T>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Argument(usize),
    Return(usize),
    Immediate(operand::Immediate),
    Temporary(operand::Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
}

impl From<i64> for Expression {
    fn from(integer: i64) -> Self {
        Expression::Immediate(operand::Immediate::Integer(integer))
    }
}

impl From<operand::Label> for Expression {
    fn from(label: operand::Label) -> Self {
        Expression::Immediate(operand::Immediate::Label(label))
    }
}

impl From<operand::Register> for Expression {
    fn from(register: operand::Register) -> Self {
        Expression::Temporary(operand::Temporary::Register(register))
    }
}

#[derive(Clone, Debug)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fallthrough;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Label(pub(crate) operand::Label);

pub trait Target: crate::data::sexp::Serialize {
    fn label(&self) -> Option<&operand::Label>;
}

impl Target for Fallthrough {
    fn label(&self) -> Option<&operand::Label> {
        None
    }
}

impl Target for Label {
    fn label(&self) -> Option<&operand::Label> {
        Some(&self.0)
    }
}
