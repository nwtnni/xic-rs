use std::fmt;

use crate::abi;
use crate::data::ir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::sexp;
use crate::data::sexp::Serialize;
use crate::data::symbol::Symbol;

pub const ZERO: Expression = Expression::Immediate(Immediate::Integer(0));
pub const ONE: Expression = Expression::Immediate(Immediate::Integer(1));
pub const EIGHT: Expression = Expression::Immediate(Immediate::Integer(abi::WORD));

pub type Unit<T> = ir::Unit<Function<T>>;

impl<T: Target> fmt::Display for Unit<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct Function<T: Target> {
    pub name: Symbol,
    pub statements: Vec<Statement<T>>,
    pub arguments: usize,
    pub returns: usize,
    pub linkage: ir::Linkage,
    pub enter: T::Access,
    pub exit: T::Access,
}

impl<T: Target> Function<T> {
    pub fn callee_arguments(&self) -> Option<usize> {
        self.statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::Call(_, arguments, _) => Some(arguments.len()),
                _ => None,
            })
            .max()
    }
}

impl<T: Target> fmt::Display for Function<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    Immediate(Immediate),
    Temporary(Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

impl From<i64> for Expression {
    fn from(integer: i64) -> Self {
        Expression::Immediate(Immediate::Integer(integer))
    }
}

impl From<Label> for Expression {
    fn from(label: Label) -> Self {
        Expression::Immediate(Immediate::Label(label))
    }
}

impl From<Temporary> for Expression {
    fn from(temporary: Temporary) -> Self {
        Expression::Temporary(temporary)
    }
}

impl From<Register> for Expression {
    fn from(register: Register) -> Self {
        Expression::Temporary(Temporary::Register(register))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Statement<T> {
    Jump(Label),
    CJump {
        condition: ir::Condition,
        left: Expression,
        right: Expression,
        r#true: Label,
        r#false: T,
    },
    Call(Expression, Vec<Expression>, usize),
    Label(Label),
    Move {
        destination: Expression,
        source: Expression,
    },
    Return(Vec<Expression>),
}

impl<T: Serialize> fmt::Display for Statement<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fallthrough;

pub trait Target: sexp::Serialize + Copy + Clone + Eq + fmt::Debug {
    type Access;
    fn target(&self) -> Option<&Label>;
    fn target_mut(&mut self) -> Option<&mut Label>;
    fn access(access: &Self::Access) -> Option<&Label>;
}

impl Target for Fallthrough {
    type Access = Label;

    fn target(&self) -> Option<&Label> {
        None
    }

    fn target_mut(&mut self) -> Option<&mut Label> {
        None
    }

    fn access(access: &Self::Access) -> Option<&Label> {
        Some(access)
    }
}

impl Target for Label {
    type Access = ();

    fn target(&self) -> Option<&Label> {
        Some(self)
    }

    fn target_mut(&mut self) -> Option<&mut Label> {
        Some(self)
    }

    fn access((): &Self::Access) -> Option<&Label> {
        None
    }
}

#[macro_export]
macro_rules! lir {
    ((CONST $($integer:tt)+)) => {
        $crate::data::lir::Expression::from(
            $crate::data::lir::lir!($($integer)+)
        )
    };
    ((NAME $($label:tt)+)) => {
        $crate::data::lir::Expression::from(
            $crate::data::lir::lir!($($label)+)
        )
    };
    ((TEMP $($temporary:tt)+)) => {
        $crate::data::lir::Expression::from(
            $crate::data::lir::lir!($($temporary)+)
        )
    };
    ((MEM $expression:tt)) => {
        $crate::data::lir::Expression::Memory(Box::new(
            $crate::data::lir::lir!($expression)
        ))
    };

    ((JUMP $label:ident)) => {
        $crate::data::lir::Statement::Jump($label)
    };
    ((CJUMP ($condition:ident $left:tt $right:tt) $r#true:ident)) => {
        $crate::data::lir::Statement::CJump {
            condition: $crate::data::ir::ir!($condition),
            left: $crate::data::lir::lir!($left),
            right: $crate::data::lir::lir!($right),
            r#true: $r#true,
            r#false: $crate::data::lir::Fallthrough,
        }
    };
    ((CJUMP ($condition:ident $left:tt $right:tt) $r#true:ident $r#false:ident)) => {
        $crate::data::lir::Statement::CJump {
            condition: $crate::data::ir::ir!($condition),
            left: $crate::data::lir::lir!($left),
            right: $crate::data::lir::lir!($right),
            r#true: $r#true,
            r#false: $r#false,
        }
    };
    ((CALL $function:tt $returns:tt $($argument:tt)*)) => {
        $crate::data::lir::Statement::Call(
            $crate::data::lir::lir!($function),
            vec![
                $(
                    $crate::data::lir::lir!($argument),
                )*
            ],
            $returns,
        )
    };
    ((LABEL $label:tt)) => {
        $crate::data::lir::Statement::Label($label)
    };
    ((MOVE $into:tt $from:tt)) => {
        $crate::data::lir::Statement::Move {
            destination: $crate::data::lir::lir!($into),
            source: $crate::data::lir::lir!($from),
        }
    };
    ((RETURN $($expression:tt)*)) => {
        $crate::data::lir::Statement::Return(
            vec![
                $(
                    $crate::data::lir::lir!($expression),
                )*
            ]
        )
    };
    (($binary:ident $left:tt $right:tt)) => {
        $crate::data::lir::Expression::Binary(
            $crate::data::ir::ir!($binary),
            Box::new($crate::data::lir::lir!($left)),
            Box::new($crate::data::lir::lir!($right)),
        )
    };

    ($expression:expr) => {
        $expression
    }
}

// https://github.com/rust-lang/rust/pull/52234#issuecomment-976702997
#[doc(hidden)]
pub use lir;
