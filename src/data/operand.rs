use std::sync::atomic::{AtomicUsize, Ordering};

use crate::util::symbol;

static LABELS: AtomicUsize = AtomicUsize::new(0);
static TEMPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Imm {
    Const(i64),
    Label(Label),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Label {
    Fix(symbol::Symbol),
    Gen(symbol::Symbol, usize),
}

impl Label {
    pub fn new(label: &'static str) -> Self {
        let idx = LABELS.fetch_add(1, Ordering::SeqCst);
        let sym = symbol::intern(label);
        Label::Gen(sym, idx)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Temp {
    Reg(Reg),
    Gen(symbol::Symbol, usize),
}

impl Temp {
    pub fn new(label: &'static str) -> Self {
        let idx = TEMPS.fetch_add(1, Ordering::SeqCst);
        let sym = symbol::intern(label);
        Temp::Gen(sym, idx)
    }
}

impl std::fmt::Display for Temp {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Reg {
    RAX,
    RBX,
    RCX,
    RDX,
    RBP,
    RSP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl std::fmt::Display for Reg {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

pub trait Operand: Copy + Eq + std::hash::Hash + std::fmt::Display + std::fmt::Debug {}
impl Operand for Temp {} 
impl Operand for Reg {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mem<T: Operand> {
    R(T),
    RO(T, i32),
}
