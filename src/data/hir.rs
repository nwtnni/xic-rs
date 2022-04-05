use std::boxed::FnBox;

use crate::data::ir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Fun {
    pub name: symbol::Symbol,
    pub body: Stm,
}

pub enum Tree {
    Ex(Exp),
    Cx(Con),
}

pub type Con = Box<dyn FnBox(operand::Label, operand::Label) -> Stm>;

impl From<Con> for Tree {
    fn from(con: Con) -> Self {
        Tree::Cx(con)
    }
}

impl From<Tree> for Con {
    fn from(tree: Tree) -> Self {
        match tree {
            Tree::Cx(con) => con,
            Tree::Ex(exp) => Box::new(move |t, f| {
                let exp = Exp::Bin(ir::Bin::Eq, Box::new(exp), Box::new(Exp::Int(1)));
                Stm::CJump(exp, t, f)
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Exp {
    Int(i64),
    Mem(Box<Exp>),
    Bin(ir::Bin, Box<Exp>, Box<Exp>),
    Name(operand::Label),
    Temp(operand::Temp),
    ESeq(Box<Stm>, Box<Exp>),
}

impl From<Exp> for Tree {
    fn from(exp: Exp) -> Self {
        Tree::Ex(exp)
    }
}

impl From<Tree> for Exp {
    fn from(tree: Tree) -> Self {
        match tree {
            Tree::Ex(exp) => exp,
            Tree::Cx(cond) => {
                let t = operand::Label::new("TRUE");
                let f = operand::Label::new("FALSE");
                let value = Exp::Temp(operand::Temp::new("BOOL"));
                let mut seq = Vec::with_capacity(5);
                seq.push(Stm::Move(value.clone(), Exp::Int(0)));
                seq.push(cond(t, f));
                seq.push(Stm::Label(t));
                seq.push(Stm::Move(value.clone(), Exp::Int(1)));
                seq.push(Stm::Label(f));
                Exp::ESeq(Box::new(Stm::Seq(seq)), Box::new(value))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Stm {
    Jump(Exp),
    CJump(Exp, operand::Label, operand::Label),
    Label(operand::Label),
    Call(Exp, Vec<Exp>),
    Move(Exp, Exp),
    Return(Vec<Exp>),
    Seq(Vec<Stm>),
}
