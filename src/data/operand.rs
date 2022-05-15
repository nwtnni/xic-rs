use std::cmp;
use std::fmt;
use std::hash::Hash;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::abi;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::util::Or;

static LABELS: AtomicUsize = AtomicUsize::new(0);
static TEMPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Immediate {
    Integer(i64),
    Label(Label),
}

impl Immediate {
    pub fn is_64_bit(&self) -> bool {
        match self {
            Immediate::Integer(integer) => i32::try_from(*integer).is_err(),
            Immediate::Label(_) => false,
        }
    }
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
    // Used for deterministic test output.
    Fixed(Symbol),
    Fresh(Symbol, usize),
    Argument(usize),
    Return(usize),
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
            Temporary::Fresh(temporary, index) => write!(fmt, "_{}{}", temporary, index),
            Temporary::Fixed(temporary) => write!(fmt, "_{}", temporary),
            Temporary::Argument(index) => write!(fmt, "_ARG{}", index),
            Temporary::Return(index) => write!(fmt, "_RET{}", index),
        }
    }
}

impl From<Register> for Temporary {
    fn from(register: Register) -> Self {
        Temporary::Register(register)
    }
}

#[derive(Copy, Clone, Debug, Eq)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rbp,
    // Note: during code generation, we sometimes need a placeholder
    // for `rsp` until we know the stack size. It must be distinguishable
    // from `rbp`, since we use `rbp` as a general-purpose register.
    //
    // In all other cases except for rewriting `Register::Rsp(false)` as an
    // offset from the stack pointer instead of the base pointer, e.g. dataflow
    // analyses, we require `Register::Rsp(true)` and `Register::Rsp(false)`
    // to be indistinguishable. Hence the custom `PartialOrd`, `Ord`, `PartialEq`,
    // and `Hash` implementations.
    Rsp(bool),
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

impl Register {
    pub const fn rsp() -> Self {
        Register::Rsp(true)
    }

    pub const fn rsp_placeholder() -> Self {
        Register::Rsp(false)
    }

    fn as_usize(&self) -> usize {
        match self {
            Register::Rax => 0,
            Register::Rbx => 1,
            Register::Rcx => 2,
            Register::Rdx => 3,
            Register::Rbp => 4,
            Register::Rsp(_) => 5,
            Register::Rsi => 6,
            Register::Rdi => 7,
            Register::R8 => 8,
            Register::R9 => 9,
            Register::R10 => 10,
            Register::R11 => 11,
            Register::R12 => 12,
            Register::R13 => 13,
            Register::R14 => 14,
            Register::R15 => 15,
        }
    }

    pub fn is_caller_saved(&self) -> bool {
        abi::CALLER_SAVED.contains(self)
    }

    pub fn is_callee_saved(&self) -> bool {
        abi::CALLEE_SAVED.contains(self)
    }
}

impl PartialEq for Register {
    fn eq(&self, other: &Self) -> bool {
        self.as_usize() == other.as_usize()
    }
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Register {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_usize().cmp(&other.as_usize())
    }
}

impl Hash for Register {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Rsp(_) => 0xdeadbeefu32.hash(state),
            _ => core::mem::discriminant(self).hash(state),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let register = match self {
            Register::Rax => "rax",
            Register::Rbx => "rbx",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::Rbp => "rbp",
            Register::Rsp(true) => "rsp",
            Register::Rsp(false) => "rbp",
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

pub trait Operand:
    Copy + Eq + Hash + std::fmt::Debug + std::fmt::Display + PartialOrd + Ord
{
}
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

impl From<Memory<Register>> for Memory<Temporary> {
    fn from(memory: Memory<Register>) -> Self {
        memory.map(|register| Temporary::from(*register))
    }
}

impl<T: Operand> From<T> for Memory<T> {
    fn from(base: T) -> Self {
        Self::B { base }
    }
}

impl<T: Operand> From<(T, T)> for Memory<T> {
    fn from((base, index): (T, T)) -> Self {
        Self::BI { base, index }
    }
}

impl<T: Operand> From<(T, T, Scale)> for Memory<T> {
    fn from((base, index, scale): (T, T, Scale)) -> Self {
        Self::BIS { base, index, scale }
    }
}

macro_rules! impl_memory_register {
    ($base:ty, $index:ty) => {
        impl From<($base, $index)> for Memory<Temporary> {
            fn from((base, index): ($base, $index)) -> Self {
                Self::BI {
                    base: base.into(),
                    index: index.into(),
                }
            }
        }

        impl From<($base, $index, Scale)> for Memory<Temporary> {
            fn from((base, index, scale): ($base, $index, Scale)) -> Self {
                Self::BIS {
                    base: base.into(),
                    index: index.into(),
                    scale,
                }
            }
        }
    };
}

impl_memory_register!(Register, Temporary);
impl_memory_register!(Temporary, Register);
impl_memory_register!(Register, Register);

macro_rules! impl_memory {
    ($immediate:ty) => {
        impl<T: Operand> From<$immediate> for Memory<T> {
            fn from(offset: $immediate) -> Self {
                Self::O {
                    offset: offset.into(),
                }
            }
        }

        impl<T: Operand> From<(T, $immediate)> for Memory<T> {
            fn from((base, offset): (T, $immediate)) -> Self {
                Self::BO {
                    base,
                    offset: offset.into(),
                }
            }
        }

        impl From<(Register, $immediate)> for Memory<Temporary> {
            fn from((base, offset): (Register, $immediate)) -> Self {
                Self::BO {
                    base: base.into(),
                    offset: offset.into(),
                }
            }
        }

        impl<T: Operand> From<(T, T, $immediate)> for Memory<T> {
            fn from((base, index, offset): (T, T, $immediate)) -> Self {
                Self::BIO {
                    base,
                    index,
                    offset: offset.into(),
                }
            }
        }

        impl<T: Operand> From<(T, Scale, $immediate)> for Memory<T> {
            fn from((index, scale, offset): (T, Scale, $immediate)) -> Self {
                Self::ISO {
                    index,
                    scale,
                    offset: offset.into(),
                }
            }
        }

        impl From<(Register, Scale, $immediate)> for Memory<Temporary> {
            fn from((index, scale, offset): (Register, Scale, $immediate)) -> Self {
                Self::ISO {
                    index: index.into(),
                    scale,
                    offset: offset.into(),
                }
            }
        }

        impl<T: Operand> From<(T, T, Scale, $immediate)> for Memory<T> {
            fn from((base, index, scale, offset): (T, T, Scale, $immediate)) -> Self {
                Self::BISO {
                    base,
                    index,
                    scale,
                    offset: offset.into(),
                }
            }
        }

        macro_rules! impl_memory_register {
            ($base:ty, $index:ty) => {
                impl From<($base, $index, $immediate)> for Memory<Temporary> {
                    fn from((base, index, offset): ($base, $index, $immediate)) -> Self {
                        Self::BIO {
                            base: base.into(),
                            index: index.into(),
                            offset: offset.into(),
                        }
                    }
                }

                impl From<($base, $index, Scale, $immediate)> for Memory<Temporary> {
                    fn from(
                        (base, index, scale, offset): ($base, $index, Scale, $immediate),
                    ) -> Self {
                        Self::BISO {
                            base: base.into(),
                            index: index.into(),
                            scale,
                            offset: offset.into(),
                        }
                    }
                }
            };
        }

        impl_memory_register!(Register, Temporary);
        impl_memory_register!(Temporary, Register);
        impl_memory_register!(Register, Register);
    };
}

impl_memory!(Immediate);
impl_memory!(Label);
impl_memory!(i64);

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
    pub fn destination(&self) -> Or<T, Memory<T>> {
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
            } => Or::L(destination.clone()),
            Binary::MI {
                destination,
                source: _,
            }
            | Binary::MR {
                destination,
                source: _,
            } => Or::R(destination.clone()),
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

macro_rules! impl_binary {
    ($immediate:ty) => {
        impl<T: Operand> From<(Memory<T>, $immediate)> for Binary<T> {
            fn from((destination, source): (Memory<T>, $immediate)) -> Self {
                Binary::MI {
                    destination,
                    source: source.into(),
                }
            }
        }

        impl<T: Operand> From<(T, $immediate)> for Binary<T> {
            fn from((destination, source): (T, $immediate)) -> Self {
                Binary::RI {
                    destination,
                    source: source.into(),
                }
            }
        }

        impl From<(Memory<Register>, $immediate)> for Binary<Temporary> {
            fn from((destination, source): (Memory<Register>, $immediate)) -> Self {
                Binary::MI {
                    destination: destination.into(),
                    source: source.into(),
                }
            }
        }

        impl From<(Register, $immediate)> for Binary<Temporary> {
            fn from((destination, source): (Register, $immediate)) -> Self {
                Binary::RI {
                    destination: destination.into(),
                    source: source.into(),
                }
            }
        }
    };
}

impl_binary!(Immediate);
impl_binary!(Label);
impl_binary!(i64);

impl<T: Operand> From<(T, Memory<T>)> for Binary<T> {
    fn from((destination, source): (T, Memory<T>)) -> Self {
        Binary::RM {
            destination,
            source,
        }
    }
}

impl<T: Operand> From<(Memory<T>, T)> for Binary<T> {
    fn from((destination, source): (Memory<T>, T)) -> Self {
        Binary::MR {
            destination,
            source,
        }
    }
}

impl<T: Operand> From<(T, T)> for Binary<T> {
    fn from((destination, source): (T, T)) -> Self {
        Binary::RR {
            destination,
            source,
        }
    }
}

macro_rules! impl_binary_register {
    ($destination:ty, $source:ty) => {
        impl From<($destination, Memory<$source>)> for Binary<Temporary> {
            fn from((destination, source): ($destination, Memory<$source>)) -> Self {
                Binary::RM {
                    destination: destination.into(),
                    source: source.into(),
                }
            }
        }

        impl From<(Memory<$destination>, $source)> for Binary<Temporary> {
            fn from((destination, source): (Memory<$destination>, $source)) -> Self {
                Binary::MR {
                    destination: destination.into(),
                    source: source.into(),
                }
            }
        }

        impl From<($destination, $source)> for Binary<Temporary> {
            fn from((destination, source): ($destination, $source)) -> Self {
                Binary::RR {
                    destination: destination.into(),
                    source: source.into(),
                }
            }
        }
    };
}

impl_binary_register!(Register, Temporary);
impl_binary_register!(Temporary, Register);
impl_binary_register!(Register, Register);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unary<T> {
    I(Immediate),
    R(T),
    M(Memory<T>),
}

impl<T> Unary<T> {
    pub fn map<F: FnMut(&T) -> U, U>(&self, mut apply: F) -> Unary<U> {
        match self {
            Unary::I(immediate) => Unary::I(*immediate),
            Unary::R(register) => Unary::R(apply(register)),
            Unary::M(memory) => Unary::M(memory.map(apply)),
        }
    }
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

impl<T> From<Memory<T>> for Unary<T> {
    fn from(memory: Memory<T>) -> Self {
        Unary::M(memory)
    }
}

impl<T> From<Immediate> for Unary<T> {
    fn from(immediate: Immediate) -> Self {
        Unary::I(immediate)
    }
}

impl<T> From<Or<T, Memory<T>>> for Unary<T> {
    fn from(operand: Or<T, Memory<T>>) -> Self {
        match operand {
            Or::L(temporary) => Unary::R(temporary),
            Or::R(memory) => Unary::M(memory),
        }
    }
}
