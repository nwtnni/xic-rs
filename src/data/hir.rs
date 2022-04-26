use crate::data::ir;
use crate::data::operand;
use crate::data::symbol;

#[derive(Clone, Debug)]
pub struct Function {
    pub name: symbol::Symbol,
    pub statements: Statement,
}

pub enum Tree {
    Expression(Expression),
    Condition(Condition),
}

pub type Condition = Box<dyn FnOnce(operand::Label, operand::Label) -> Statement>;

impl From<Condition> for Tree {
    fn from(condition: Condition) -> Self {
        Tree::Condition(condition)
    }
}

impl From<Tree> for Condition {
    fn from(tree: Tree) -> Self {
        match tree {
            Tree::Condition(condition) => condition,
            Tree::Expression(expression) => Box::new(move |r#true, r#false| {
                let condition = Expression::Binary(
                    ir::Binary::Eq,
                    Box::new(expression),
                    Box::new(Expression::Integer(1)),
                );
                Statement::CJump(condition, r#true, r#false)
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
    ((CJUMP $condition:tt $r#true:ident $r#false:ident)) => {
        crate::data::hir::Statement::CJump(
            hir!($condition),
            $r#true,
            $r#false,
        )
    };
    ((LABEL $label:tt)) => {
        crate::data::hir::Statement::Label($label)
    };
    ((EXP $expression:tt)) => {
        crate::data::hir::Statement::Expression(hir!($expression))
    };
    ((MOVE $into:tt $from:tt)) => {
        crate::data::hir::Statement::Move(
            hir!($into),
            hir!($from),
        )
    };
    ((RETURN)) => {
        crate::data::hir::Statement::Return
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

#[derive(Clone, Debug)]
pub enum Expression {
    Integer(i64),
    Label(operand::Label),
    Temporary(operand::Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>, usize),
    Sequence(Box<Statement>, Box<Expression>),
}

impl From<operand::Temporary> for Expression {
    fn from(temporary: operand::Temporary) -> Self {
        Self::Temporary(temporary)
    }
}

impl From<operand::Label> for Expression {
    fn from(label: operand::Label) -> Self {
        Self::Label(label)
    }
}

impl From<i64> for Expression {
    fn from(integer: i64) -> Self {
        Self::Integer(integer)
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
                let r#true = operand::Label::fresh("true");
                let r#false = operand::Label::fresh("false");
                let value = Expression::Temporary(operand::Temporary::fresh("bool"));

                let sequence = vec![
                    Statement::Move(value.clone(), Expression::Integer(0)),
                    condition(r#true, r#false),
                    Statement::Label(r#true),
                    Statement::Move(value.clone(), Expression::Integer(1)),
                    Statement::Label(r#false),
                ];

                Expression::Sequence(Box::new(Statement::Sequence(sequence)), Box::new(value))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    Jump(operand::Label),
    CJump(Expression, operand::Label, operand::Label),
    Label(operand::Label),
    Expression(Expression),
    Move(Expression, Expression),
    Return,
    Sequence(Vec<Statement>),
}
