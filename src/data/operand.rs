use std::sync::atomic::{AtomicUsize, Ordering};

use crate::data::symbol;

static LABELS: AtomicUsize = AtomicUsize::new(0);
static TEMPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Immediate {
    Constant(i64),
    Label(Label),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Label {
    Fixed(symbol::Symbol),
    Fresh(symbol::Symbol, usize),
}

impl Label {
    pub fn fresh(label: &'static str) -> Self {
        let index = LABELS.fetch_add(1, Ordering::SeqCst);
        let symbol = symbol::intern_static(label);
        Label::Fresh(symbol, index)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Temporary {
    Register(Register),
    Argument(usize),
    Return(usize),
    Fresh(symbol::Symbol, usize),
}

impl Temporary {
    pub fn fresh(label: &'static str) -> Self {
        let index = TEMPS.fetch_add(1, Ordering::SeqCst);
        let symbol = symbol::intern_static(label);
        Temporary::Fresh(symbol, index)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rbp,
    Rsp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

pub trait Operand: Copy + Eq + std::hash::Hash + std::fmt::Debug + PartialOrd + Ord {}
impl Operand for Temporary {}
impl Operand for Register {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Memory<T: Operand> {
    R(T),
    RO(T, i32),
}
