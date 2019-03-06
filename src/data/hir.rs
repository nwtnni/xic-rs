use std::collections::HashMap;

use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Unit {
    pub name: symbol::Symbol,
    pub funs: HashMap<operand::Label, Fun>,
}

#[derive(Clone, Debug)]
pub struct Fun {
    pub name: operand::Label,
    pub body: Stm,
}

#[derive(Clone, Debug)]
pub enum Exp {
    Int(i64), 
    Mem(Box<Exp>),
    Bin(Bin, Box<Exp>, Box<Exp>),    
    Name(operand::Label),
    Call(Box<Exp>, Vec<Exp>),
    Temp(operand::Temp),
    ESeq(Box<Stm>, Box<Exp>),
}

#[derive(Clone, Debug)]
pub enum Stm {
    Exp(Exp),
    Jump(Exp),
    CJump(Exp, operand::Label, operand::Label),
    Label(operand::Label),
    Move(Exp, Exp),
    Return(Vec<Exp>),
    Seq(Vec<Stm>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bin {
    Add,
    Sub,
    Mul,
    Hul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Ls,
    Rs,
    ARs,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rel {
    Lt,
    Le,
    Ge,
    Gt,
    Ne,
    Eq,
}
