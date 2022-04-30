use std::fmt;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::data::symbol;
use crate::data::symbol::Symbol;

static LABELS: AtomicUsize = AtomicUsize::new(0);
static TEMPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Immediate {
    Integer(i64),
    Label(Label),
}

impl fmt::Display for Immediate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Immediate::Integer(integer) => write!(fmt, "{}", integer),
            Immediate::Label(label) => write!(fmt, "{}", label),
        }
    }
}

impl From<i64> for Immediate {
    fn from(integer: i64) -> Self {
        Immediate::Integer(integer)
    }
}

impl From<Label> for Immediate {
    fn from(label: Label) -> Self {
        Immediate::Label(label)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Label {
    Fixed(Symbol),
    Fresh(Symbol, usize),
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
    Fresh(Symbol, usize),
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
            Temporary::Fresh(temporary, index) => write!(fmt, "{}{}", temporary, index),
        }
    }
}

impl From<Register> for Temporary {
    fn from(register: Register) -> Self {
        Temporary::Register(register)
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

impl<T> Memory<T> {
    pub fn map<F: FnMut(&T) -> U, U>(&self, mut apply: F) -> Memory<U> {
        match self {
            Memory::B { base } => Memory::B { base: apply(base) },
            Memory::O { offset } => Memory::O { offset: *offset },
            Memory::BI { base, index } => Memory::BI {
                base: apply(base),
                index: apply(index),
            },
            Memory::BO { base, offset } => Memory::BO {
                base: apply(base),
                offset: *offset,
            },
            Memory::BIO {
                base,
                index,
                offset,
            } => Memory::BIO {
                base: apply(base),
                index: apply(index),
                offset: *offset,
            },
            Memory::BIS { base, index, scale } => Memory::BIS {
                base: apply(base),
                index: apply(index),
                scale: *scale,
            },
            Memory::ISO {
                index,
                scale,
                offset,
            } => Memory::ISO {
                index: apply(index),
                scale: *scale,
                offset: *offset,
            },
            Memory::BISO {
                base,
                index,
                scale,
                offset,
            } => Memory::BISO {
                base: apply(base),
                index: apply(index),
                scale: *scale,
                offset: *offset,
            },
        }
    }
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
pub enum Binary<T> {
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

impl<T: Clone> Binary<T> {
    pub fn destination(&self) -> Unary<T> {
        match self {
            Binary::RI {
                destination,
                source: _,
            }
            | Binary::RM {
                destination,
                source: _,
            }
            | Binary::RR {
                destination,
                source: _,
            } => Unary::R(destination.clone()),
            Binary::MI {
                destination,
                source: _,
            }
            | Binary::MR {
                destination,
                source: _,
            } => Unary::M(destination.clone()),
        }
    }

    pub fn source(&self) -> Unary<T> {
        match self {
            Binary::RI {
                destination: _,
                source,
            }
            | Binary::MI {
                destination: _,
                source,
            } => Unary::I(*source),
            Binary::MR {
                destination: _,
                source,
            }
            | Binary::RR {
                destination: _,
                source,
            } => Unary::R(source.clone()),
            Binary::RM {
                destination: _,
                source,
            } => Unary::M(source.clone()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unary<T> {
    I(Immediate),
    R(T),
    M(Memory<T>),
}

impl<T> From<i64> for Unary<T> {
    fn from(integer: i64) -> Self {
        Unary::I(Immediate::Integer(integer))
    }
}

impl<T> From<Label> for Unary<T> {
    fn from(label: Label) -> Self {
        Unary::I(Immediate::Label(label))
    }
}

impl<T: From<Register>> From<Register> for Unary<T> {
    fn from(register: Register) -> Self {
        Unary::R(T::from(register))
    }
}

impl<T: From<Temporary>> From<Temporary> for Unary<T> {
    fn from(temporary: Temporary) -> Self {
        Unary::R(T::from(temporary))
    }
}
