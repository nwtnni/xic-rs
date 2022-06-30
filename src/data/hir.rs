use std::fmt;

use crate::data::ir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::sexp::Serialize as _;
use crate::data::symbol::Symbol;

pub const ZERO: Expression = Expression::Immediate(Immediate::Integer(0));
pub const ONE: Expression = Expression::Immediate(Immediate::Integer(1));

pub type Unit = ir::Unit<Function>;

impl fmt::Display for Unit {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: Symbol,
    pub statement: Statement,
    pub arguments: Vec<Temporary>,
    pub returns: usize,
    pub linkage: ir::Linkage,
}

impl fmt::Display for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

pub enum Tree {
    Expression(Expression),
    Condition(Condition),
}

pub type Condition = Box<dyn FnOnce(Label, Label) -> Statement>;

impl From<Condition> for Tree {
    fn from(condition: Condition) -> Self {
        Tree::Condition(condition)
    }
}

impl From<Tree> for Condition {
    fn from(tree: Tree) -> Self {
        match tree {
            Tree::Condition(condition) => condition,
            Tree::Expression(expression) => Box::new(move |r#true, r#false| Statement::CJump {
                condition: ir::Condition::Eq,
                left: expression,
                right: Expression::from(1),
                r#true,
                r#false,
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Immediate(Immediate),
    Temporary(Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>, Vec<Temporary>),
    Sequence(Box<Statement>, Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

impl From<Temporary> for Expression {
    fn from(temporary: Temporary) -> Self {
        Self::Temporary(temporary)
    }
}

impl<T: Into<Immediate>> From<T> for Expression {
    fn from(immediate: T) -> Self {
        Self::Immediate(immediate.into())
    }
}

impl From<Expression> for Tree {
    fn from(expression: Expression) -> Self {
        Tree::Expression(expression)
    }
}

impl From<Tree> for Expression {
    fn from(tree: Tree) -> Self {
        match tree {
            Tree::Expression(expression) => expression,
            Tree::Condition(condition) => {
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");
                let value = Expression::Temporary(Temporary::fresh("bool"));

                let sequence = vec![
                    Statement::Move {
                        destination: value.clone(),
                        source: Expression::from(0),
                    },
                    condition(r#true, r#false),
                    Statement::Label(r#true),
                    Statement::Move {
                        destination: value.clone(),
                        source: Expression::from(1),
                    },
                    Statement::Label(r#false),
                ];

                Expression::Sequence(Box::new(Statement::Sequence(sequence)), Box::new(value))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement {
    Jump(Label),
    CJump {
        condition: ir::Condition,
        left: Expression,
        right: Expression,
        r#true: Label,
        r#false: Label,
    },
    Label(Label),
    Expression(Expression),
    Move {
        destination: Expression,
        source: Expression,
    },
    Return(Vec<Expression>),
    Sequence(Vec<Statement>),
}

impl fmt::Display for Statement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[macro_export]
macro_rules! hir {
    ((CONST $($integer:tt)+)) => {
        $crate::data::hir::Expression::from(
            $crate::data::hir::hir!($($integer)+)
        )
    };
    ((NAME $($label:tt)+)) => {
        $crate::data::hir::Expression::from(
            $crate::data::hir::hir!($($label)+)
        )
    };
    ((TEMP $($temporary:tt)+)) => {
        $crate::data::hir::Expression::from(
            $crate::data::hir::hir!($($temporary)+)
        )
    };
    ((MEM $expression:tt)) => {
        $crate::data::hir::Expression::Memory(Box::new(
            $crate::data::hir::hir!($expression)
        ))
    };
    ((CALL $function:tt $returns:tt $($argument:tt)*)) => {
        $crate::data::hir::Expression::Call(
            Box::new(
                $crate::data::hir::hir!($function)
            ),
            vec![
                $(
                    $crate::data::hir::hir!($argument),
                )*
            ],
            $returns,
        )
    };
    ((ESEQ $statement:tt $expression:tt)) => {
        $crate::data::hir::Expression::Sequence(
            Box::new($crate::data::hir::hir!($statement)),
            Box::new($crate::data::hir::hir!($expression)),
        )
    };

    ((JUMP $label:ident)) => {
        $crate::data::hir::Statement::Jump($label)
    };
    ((CJUMP ($condition:ident $left:tt $right:tt) $r#true:ident $r#false:ident)) => {
        $crate::data::hir::Statement::CJump {
            condition: $crate::data::ir::ir!($condition),
            left: $crate::data::hir::hir!($left),
            right: $crate::data::hir::hir!($right),
            r#true: $r#true,
            r#false: $r#false,
        }
    };
    ((LABEL $label:tt)) => {
        $crate::data::hir::Statement::Label($label)
    };
    ((EXP $expression:tt)) => {
        $crate::data::hir::Statement::Expression(
            $crate::data::hir::hir!($expression)
        )
    };
    ((MOVE $into:tt $from:tt)) => {
        $crate::data::hir::Statement::Move {
            destination: $crate::data::hir::hir!($into),
            source: $crate::data::hir::hir!($from),
        }
    };
    ((RETURN $($expression:tt)*)) => {
        $crate::data::hir::Statement::Return(
            vec![
                $(
                    $crate::data::hir::hir!($expression),
                )*
            ]
        )
    };
    ((SEQ $($statement:tt)*)) => {
        $crate::data::hir::Statement::Sequence(
            vec![
                $(
                    $crate::data::hir::hir!($statement),
                )*
            ],
        )
    };
    (($binary:ident $left:tt $right:tt)) => {
        $crate::data::hir::Expression::Binary(
            $crate::data::ir::ir!($binary),
            Box::new($crate::data::hir::hir!($left)),
            Box::new($crate::data::hir::hir!($right)),
        )
    };

    ($expression:expr) => {
        $expression
    }
}

// https://github.com/rust-lang/rust/pull/52234#issuecomment-976702997
#[doc(hidden)]
pub use hir;
