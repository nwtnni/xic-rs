use crate::abi;
use crate::data::ir;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Register;
use crate::data::operand::Temporary;

pub const ZERO: Expression = Expression::Immediate(Immediate::Integer(0));
pub const ONE: Expression = Expression::Immediate(Immediate::Integer(1));
pub const EIGHT: Expression = Expression::Immediate(Immediate::Integer(abi::WORD));

pub type Unit<T> = ir::Unit<Function<T>>;
pub type Function<T> = ir::Function<Vec<Statement<T>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Argument(usize),
    Return(usize),
    Immediate(Immediate),
    Temporary(Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
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
