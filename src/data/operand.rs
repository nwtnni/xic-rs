use std::fmt;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::data::symbol;

static LABELS: AtomicUsize = AtomicUsize::new(0);
static TEMPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Immediate {
    Constant(i64),
    Label(Label),
}

impl fmt::Display for Immediate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Immediate::Constant(integer) => write!(fmt, "{:0x}", integer),
            Immediate::Label(label) => write!(fmt, "{}", label),
        }
    }
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

impl fmt::Display for Label {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Label::Fixed(label) => write!(fmt, "{}", label),
            Label::Fresh(label, index) => {
                write!(fmt, "{}{}", label, index)
            }
        }
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

impl fmt::Display for Temporary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Temporary::Register(register) => write!(fmt, "{}", register),
            Temporary::Argument(index) => write!(fmt, "_ARG{}", index),
            Temporary::Return(index) => write!(fmt, "_RET{}", index),
            Temporary::Fresh(temporary, index) => write!(fmt, "{}{}", temporary, index),
        }
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

impl fmt::Display for Register {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let register = match self {
            Register::Rax => "rax",
            Register::Rbx => "rbx",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::Rbp => "rbp",
            Register::Rsp => "rsp",
            Register::Rsi => "rsi",
            Register::Rdi => "rdi",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
        };

        write!(fmt, "{}", register)
    }
}

pub trait Operand: Copy + Eq + std::hash::Hash + std::fmt::Debug + PartialOrd + Ord {}
impl Operand for Temporary {}
impl Operand for Register {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Memory<T> {
    B {
        base: T,
    },
    O {
        offset: Immediate,
    },
    BI {
        base: T,
        index: T,
    },
    BO {
        base: T,
        offset: Immediate,
    },
    BIO {
        base: T,
        index: T,
        offset: Immediate,
    },
    BIS {
        base: T,
        index: T,
        scale: Scale,
    },
    ISO {
        index: T,
        scale: Scale,
        offset: Immediate,
    },
    BISO {
        base: T,
        index: T,
        scale: Scale,
        offset: Immediate,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scale {
    _1,
    _2,
    _4,
    _8,
}

impl fmt::Display for Scale {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let scale = match self {
            Scale::_1 => '1',
            Scale::_2 => '2',
            Scale::_4 => '4',
            Scale::_8 => '8',
        };

        write!(fmt, "{}", scale)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Two<T> {
    RI {
        destination: T,
        source: Immediate,
    },
    MI {
        destination: Memory<T>,
        source: Immediate,
    },
    MR {
        destination: Memory<T>,
        source: T,
    },
    RM {
        destination: T,
        source: Memory<T>,
    },
    RR {
        destination: T,
        source: T,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum One<T> {
    I(Immediate),
    R(T),
    M(Memory<T>),
}
