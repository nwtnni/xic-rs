use crate::data::ir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Fun {
    pub name: symbol::Symbol,
    pub body: Vec<Stm>,
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
    Jump(Exp),
    CJump(Exp, operand::Label, operand::Label),
    Call(Exp, Vec<Exp>),
    Label(operand::Label),
    Move(Exp, Exp),
    Return(Vec<Exp>),
}
