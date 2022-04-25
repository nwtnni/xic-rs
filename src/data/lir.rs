use crate::data::ir;
use crate::data::operand;
use crate::data::symbol;

#[derive(Clone, Debug)]
pub struct Function {
    pub name: symbol::Symbol,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Integer(i64),
    Label(operand::Label),
    Temporary(operand::Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum Statement {
    Jump(operand::Label),
    CJump(Expression, operand::Label, operand::Label),
    Call(Expression, Vec<Expression>, usize),
    Label(operand::Label),
    Move(Expression, Expression),
    Return(Vec<Expression>),
}
