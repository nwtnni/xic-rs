use crate::data::symbol::Symbol;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    Any,
    Integer,
    Boolean,
    Class(Symbol),
    Array(Box<Expression>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::Any => write!(fmt, "any"),
            Expression::Class(class) => write!(fmt, "{}", class),
            Expression::Integer => write!(fmt, "int"),
            Expression::Boolean => write!(fmt, "bool"),
            Expression::Array(typ) => write!(fmt, "{}[]", typ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Statement {
    Unit,
    Void,
}

impl Statement {
    pub fn least_upper_bound(&self, other: &Statement) -> Statement {
        match (self, other) {
            (Statement::Void, Statement::Void) => Statement::Void,
            _ => Statement::Unit,
        }
    }
}
