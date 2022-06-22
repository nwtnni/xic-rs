use crate::data::symbol::Symbol;

/// ```text
///            .  .   .
///            .  .   .
///            .  .   .
///            |  |   |
///            A  B   C         A[] B[] C[]
///             \ |  /            \  |  /
/// int   bool   null int[] bool[] null[] int[][] bool[][]
///  |     |      |     |     |      |       |      |
///  |     |      |     |     |      |       |      |
///  +-----+----< any >-+-----+------+-------+------+---- . . .
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    /// The inner type of an empty array, which subclasses everything.
    Any,
    /// The type of a `null` expression, which subclasses any class.
    Null,
    Integer,
    Boolean,
    Class(Symbol),
    Array(Box<Expression>),
    Function(Vec<Expression>, Vec<Expression>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::Any => write!(fmt, "any"),
            Expression::Null => write!(fmt, "null"),
            Expression::Class(class) => write!(fmt, "{}", class),
            Expression::Integer => write!(fmt, "int"),
            Expression::Boolean => write!(fmt, "bool"),
            Expression::Array(typ) => write!(fmt, "{}[]", typ),
            Expression::Function(parameters, returns) => {
                write!(fmt, "fn(")?;

                let mut parameters = parameters.iter();

                if let Some(head) = parameters.next() {
                    write!(fmt, "{}", head)?;
                }

                for parameter in parameters {
                    write!(fmt, ", {}", parameter)?;
                }

                write!(fmt, ")")?;

                let mut returns = returns.iter();

                if let Some(r#return) = returns.next() {
                    write!(fmt, ": {}", r#return)?;
                }

                for r#return in returns {
                    write!(fmt, ", {}", r#return)?;
                }

                Ok(())
            }
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
