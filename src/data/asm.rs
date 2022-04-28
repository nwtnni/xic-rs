use crate::data::operand;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Assembly<T> {
    Binary(Binary, operand::Two<T>),
    Unary(Unary, operand::One<T>),
    Nullary(Nullary),
    Directive(Directive),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Directive {
    Align(usize),
    Local(operand::Label),
    Global(operand::Label),
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unary {
    Neg,
    Push,
    Pop,
    Call,
    Mul,
    Div(Division),
    Jmp,
    Jcc(Condition),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nullary {
    Cqo,
    Ret,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    E,
    Ne,
    G,
    Ge,
    L,
    Le,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Division {
    Quotient,
    Remainder,
}
