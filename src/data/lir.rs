use std::collections::HashMap;

use crate::data::ir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Fun {
    pub name: operand::Label,
    pub body: Vec<Stm>,
    pub vars: HashMap<symbol::Symbol, operand::Temp>,
}

#[derive(Clone, Debug)]
pub enum Exp {
    Int(i64), 
    Mem(Box<Exp>),
    Bin(ir::Bin, Box<Exp>, Box<Exp>),    
    Name(operand::Label),
    Temp(operand::Temp),
}

#[derive(Clone, Debug)]
pub enum Stm {
    Exp(Exp),
    Jump(Exp),
    CJump(ir::Rel, Exp, Exp, operand::Label),
    Call(Box<Exp>, Vec<Exp>),
    Label(operand::Label),
    Move(Exp, Exp),
    Return(Vec<Exp>),
    Seq(Vec<Stm>),
}
