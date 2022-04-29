use crate::data::ir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;

pub const ZERO: Expression = Expression::Immediate(Immediate::Integer(0));
pub const ONE: Expression = Expression::Immediate(Immediate::Integer(1));

pub type Unit = ir::Unit<Function>;
pub type Function = ir::Function<Statement>;

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

#[macro_export]
macro_rules! hir {
    ((CONST $($integer:tt)+)) => {
        crate::data::hir::Expression::from(hir!($($integer)+))
    };
    ((NAME $($label:tt)+)) => {
        crate::data::hir::Expression::from(hir!($($label)+))
    };
    ((TEMP $($temporary:tt)+)) => {
        crate::data::hir::Expression::from(hir!($($temporary)+))
    };
    ((MEM $expression:tt)) => {
        crate::data::hir::Expression::Memory(Box::new(hir!($expression)))
    };
    ((CALL $function:tt $returns:tt $($argument:tt)*)) => {
        crate::data::hir::Expression::Call(
            Box::new(hir!($function)),
            vec![$(hir!($argument),)*],
            $returns,
        )
    };
    ((ESEQ $statement:tt $expression:tt)) => {
        crate::data::hir::Expression::Sequence(
            Box::new(hir!($statement)),
            Box::new(hir!($expression)),
        )
    };

    ((JUMP $label:ident)) => {
        crate::data::hir::Statement::Jump($label)
    };
    ((CJUMP ($condition:ident $left:tt $right:tt) $r#true:ident $r#false:ident)) => {
        crate::data::hir::Statement::CJump {
            condition: $condition,
            left: hir!($left),
            right: hir!($right),
            r#true: $r#true,
            r#false: $r#false,
        }
    };
    ((LABEL $label:tt)) => {
        crate::data::hir::Statement::Label($label)
    };
    ((EXP $expression:tt)) => {
        crate::data::hir::Statement::Expression(hir!($expression))
    };
    ((MOVE $into:tt $from:tt)) => {
        crate::data::hir::Statement::Move {
            destination: hir!($into),
            source: hir!($from),
        }
    };
    ((RETURN $returns:expr)) => {
        crate::data::hir::Statement::Return($returns)
    };
    ((SEQ $statement:tt $($statements:tt)+)) => {
        crate::data::hir::Statement::Sequence(vec![
            hir!($statement),
            $(hir!($statements),)*
        ])
    };
    ((SEQ $statements:expr)) => {
        crate::data::hir::Statement::Sequence($statements)
    };

    (($binary:ident $left:tt $right:tt)) => {
        crate::data::hir::Expression::Binary(
            $binary,
            Box::new(hir!($left)),
            Box::new(hir!($right)),
        )
    };
    ($expression:expr) => {
        $expression
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Argument(usize),
    Return(usize),
    Immediate(Immediate),
    Temporary(Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>, usize),
    Sequence(Box<Statement>, Box<Expression>),
}

impl From<Temporary> for Expression {
    fn from(temporary: Temporary) -> Self {
        Self::Temporary(temporary)
    }
}

impl From<Label> for Expression {
    fn from(label: Label) -> Self {
        Self::Immediate(Immediate::Label(label))
    }
}

impl From<i64> for Expression {
    fn from(integer: i64) -> Self {
        Self::Immediate(Immediate::Integer(integer))
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
