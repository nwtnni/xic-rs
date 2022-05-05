use std::fmt;

use crate::data::ir;
use crate::data::operand;
use crate::data::operand::Label;
use crate::data::symbol::Symbol;

pub type Unit<T> = ir::Unit<Function<T>>;

impl<T: fmt::Display> fmt::Display for Unit<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", crate::assemble::Intel(self))
    }
}

impl<T: fmt::Display> Unit<T> {
    pub fn intel(&self) -> impl fmt::Display + '_ {
        crate::assemble::Intel(self)
    }
}

#[derive(Clone, Debug)]
pub struct Function<T> {
    pub name: Symbol,
    pub statements: Vec<Statement<T>>,
    pub arguments: usize,
    pub returns: usize,
    pub callee_arguments: usize,
    pub callee_returns: usize,
    pub enter: Label,
    pub exit: Label,
}

impl<T: fmt::Display> Function<T> {
    pub fn intel(&self) -> impl fmt::Display + '_ {
        crate::assemble::Intel(self)
    }
}

impl<T: fmt::Display> fmt::Display for Function<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", crate::assemble::Intel(self))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Statement<T> {
    Binary(Binary, operand::Binary<T>),
    Unary(Unary, operand::Unary<T>),
    Nullary(Nullary),
    Label(Label),
    Jmp(Label),
    Jcc(Condition, Label),
}

impl<T: fmt::Display> Statement<T> {
    pub fn intel(&self) -> impl fmt::Display + '_ {
        crate::assemble::Intel(self)
    }
}

impl<T: fmt::Display> fmt::Display for Statement<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", crate::assemble::Intel(self))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Directive {
    Intel,
    Align(usize),
    Local(Label),
    Global(Label),
    Quad(Vec<i64>),
    Data,
    Text,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Binary {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Cmp,
    Mov,
    Lea,
    Shl,
}

impl From<ir::Binary> for Binary {
    fn from(binary: ir::Binary) -> Self {
        match binary {
            ir::Binary::Add => Binary::Add,
            ir::Binary::Sub => Binary::Sub,
            ir::Binary::Xor => Binary::Xor,
            ir::Binary::And => Binary::And,
            ir::Binary::Or => Binary::Or,
            _ => panic!("[INTERNAL ERROR]: converting unsupported IR operator"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unary {
    Neg,
    Call { arguments: usize, returns: usize },
    Mul,
    Hul,
    Div,
    Mod,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nullary {
    Nop,
    Cqo,
    Ret(usize),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    L,
    Le,
    Ge,
    G,
    Ne,
    E,
    Ae,
}

impl From<ir::Condition> for Condition {
    fn from(condition: ir::Condition) -> Self {
        match condition {
            ir::Condition::Lt => Condition::L,
            ir::Condition::Le => Condition::Le,
            ir::Condition::Ge => Condition::Ge,
            ir::Condition::Gt => Condition::G,
            ir::Condition::Ne => Condition::Ne,
            ir::Condition::Eq => Condition::E,
            ir::Condition::Ae => Condition::Ae,
        }
    }
}

#[macro_export]
macro_rules! asm {
    (($label:ident:)) => {
        $crate::data::asm::Statement::Label($label)
    };
    ((jmp $label:expr)) => {
        $crate::data::asm::Statement::Jmp($label)
    };
    ((jcc $condition:expr, $label:expr)) => {
        $crate::data::asm::Statement::Jcc($condition, $label)
    };
    ((nop)) => {
        $crate::data::asm::Statement::Nullary($crate::data::asm::Nullary::Nop)
    };
    ((cqo)) => {
        $crate::data::asm::Statement::Nullary($crate::data::asm::Nullary::Cqo)
    };
    ((ret<$returns:tt>)) => {
        $crate::data::asm::Statement::Nullary($crate::data::asm::Nullary::Ret(
            $returns,
        ))
    };

    ((call<$arguments:tt, $returns:tt> $operand:tt)) => {
        $crate::data::asm::Statement::Unary(
            $crate::data::asm::Unary::Call {
                arguments: $arguments,
                returns: $returns,
            },
            $crate::data::operand::Unary::from(asm!($operand))
        )
    };
    (($unary:tt $operand:tt)) => {
        $crate::data::asm::Statement::Unary(
            asm!(@unary $unary),
            $crate::data::operand::Unary::from(asm!($operand))
        )
    };
    (($binary:tt $destination:tt, $source:tt)) => {
        $crate::data::asm::Statement::Binary(
            asm!(@binary $binary),
            $crate::data::operand::Binary::from((asm!($destination), asm!($source)))
        )
    };

    (@unary neg) => { $crate::data::asm::Unary::Neg };
    (@unary imul) => { $crate::data::asm::Unary::Mul };
    (@unary ihul) => { $crate::data::asm::Unary::Hul };
    (@unary idiv) => { $crate::data::asm::Unary::Div };
    (@unary imod) => { $crate::data::asm::Unary::Mod };

    (@binary add) => { $crate::data::asm::Binary::Add };
    (@binary sub) => { $crate::data::asm::Binary::Sub };
    (@binary and) => { $crate::data::asm::Binary::And };
    (@binary shl) => { $crate::data::asm::Binary::Shl };
    (@binary or) => { $crate::data::asm::Binary::Or };
    (@binary xor) => { $crate::data::asm::Binary::Xor };
    (@binary cmp) => { $crate::data::asm::Binary::Cmp };
    (@binary mov) => { $crate::data::asm::Binary::Mov };
    (@binary lea) => { $crate::data::asm::Binary::Lea };

    ([$base:tt]) => {
        $crate::data::operand::Memory::from($base)
    };
    ([$base:tt + $offset:tt]) => {
        $crate::data::operand::Memory::from(($base, $offset))
    };
    ([$base:tt + $index:tt + $offset:tt]) => {
        $crate::data::operand::Memory::from(($base, $index, $offset))
    };
    ([$base:tt + $index:tt * $scale:tt]) => {
        $crate::data::operand::Memory::from(($base, $index, $scale))
    };
    ([$index:tt * $scale:tt + $offset:tt]) => {
        $crate::data::operand::Memory::from(($index, $scale, $offset))
    };
    ([$base:tt + $index:tt * $scale:tt + $offset:tt]) => {
        $crate::data::operand::Memory::from(($base, $index, $scale, $offset))
    };

    ($tt:tt) => {
        $tt
    };
}
