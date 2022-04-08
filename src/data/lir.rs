use crate::data::ir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Function {
    pub name: symbol::Symbol,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Int(i64),
    Mem(Box<Expression>),
    Bin(ir::Bin, Box<Expression>, Box<Expression>),
    Name(operand::Label),
    Temp(operand::Temp),
}

#[derive(Clone, Debug)]
pub enum Statement {
    Jump(Expression),
    CJump(Expression, operand::Label, operand::Label),
    Call(Expression, Vec<Expression>),
    Label(operand::Label),
    Move(Expression, Expression),
    Return(Vec<Expression>),
}
