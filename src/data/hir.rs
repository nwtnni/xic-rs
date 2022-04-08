use crate::data::ir;
use crate::data::operand;
use crate::util::symbol;

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

#[derive(Clone, Debug)]
pub enum Expression {
    Integer(i64),
    Label(operand::Label),
    Temporary(operand::Temporary),
    Memory(Box<Expression>),
    Binary(ir::Binary, Box<Expression>, Box<Expression>),
    Call(Call),
    Sequence(Box<Statement>, Box<Expression>),
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
    Jump(Expression),
    CJump(Expression, operand::Label, operand::Label),
    Label(operand::Label),
    Call(Call),
    Move(Expression, Expression),
    Return(Vec<Expression>),
    Sequence(Vec<Statement>),
}

#[derive(Clone, Debug)]
pub struct Call {
    pub name: Box<Expression>,
    pub arguments: Vec<Expression>,
}
