use std::boxed::FnBox;
use std::collections::HashMap;

use crate::data::ir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub struct Fun {
    pub name: operand::Label,
    pub body: Stm,
    pub vars: HashMap<symbol::Symbol, operand::Temp>,
}

pub enum Tree {
    Nx(Stm),
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
        | Tree::Nx(_) => unreachable!(),
        | Tree::Cx(con) => con,
        | Tree::Ex(exp) => {
            Box::new(move |t, f| {
                Stm::CJump(ir::Rel::Eq, Exp::Int(1), exp, t, f)
            })
        }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Exp {
    Int(i64), 
    Mem(Box<Exp>),
    Bin(ir::Bin, Box<Exp>, Box<Exp>),    
    Name(operand::Label),
    Call(Box<Exp>, Vec<Exp>),
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
        | Tree::Nx(stm) => Exp::ESeq(Box::new(stm), Box::new(Exp::Int(0))),
        | Tree::Ex(exp) => exp,
        | Tree::Cx(cond) => {
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
    Exp(Exp),
    Jump(Exp),
    CJump(ir::Rel, Exp, Exp, operand::Label, operand::Label),
    Label(operand::Label),
    Move(Exp, Exp),
    Return(Vec<Exp>),
    Seq(Vec<Stm>),
}

impl From<Stm> for Tree {
    fn from(stm: Stm) -> Self {
        Tree::Nx(stm)
    }
}

impl From<Tree> for Stm {
    fn from(tree: Tree) -> Self {
        match tree {
        | Tree::Nx(stm) => stm,
        | Tree::Ex(exp) => Stm::Exp(exp),
        | Tree::Cx(cond) => {
            let label = operand::Label::new("STM");
            (cond)(label, label)
        }
        }
    }
}
