use crate::data::ir;
use crate::data::operand;
use crate::data::symbol;

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
