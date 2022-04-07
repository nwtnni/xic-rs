#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    Integer,
    Boolean,
    Any,
    Array(Box<Expression>),
}

impl Expression {
    pub fn subtypes(&self, rhs: &Expression) -> bool {
        match (self, rhs) {
            (Expression::Any, _)
            | (Expression::Integer, Expression::Integer)
            | (Expression::Boolean, Expression::Boolean) => true,
            (Expression::Array(t), Expression::Array(_)) if **t == Expression::Any => true,
            (Expression::Array(l), Expression::Array(r)) => l.subtypes(r),
            _ => false,
        }
    }

    pub fn least_upper_bound(&self, rhs: &Expression) -> Option<Self> {
        match (self, rhs) {
            (Expression::Any, typ) | (typ, Expression::Any) => Some(typ.clone()),
            (Expression::Integer, Expression::Integer) => Some(Expression::Integer),
            (Expression::Boolean, Expression::Boolean) => Some(Expression::Boolean),
            (Expression::Array(lhs), Expression::Array(rhs)) => lhs
                .least_upper_bound(rhs)
                .map(Box::new)
                .map(Expression::Array),
            (_, _) => None,
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::Integer => write!(fmt, "int"),
            Expression::Boolean => write!(fmt, "bool"),
            Expression::Any => write!(fmt, "any"), // Panic?
            Expression::Array(typ) => write!(fmt, "{}[]", typ),
        }
    }
}

pub fn subtypes<'a, 'b, L, R>(subtypes: L, supertypes: R) -> bool
where
    L: IntoIterator<Item = &'a Expression>,
    R: IntoIterator<Item = &'b Expression>,
{
    let mut subtypes = subtypes.into_iter();
    let mut supertypes = supertypes.into_iter();
    loop {
        match (subtypes.next(), supertypes.next()) {
            (None, None) => return true,
            (None, Some(_)) | (Some(_), None) => return false,
            (Some(subtype), Some(supertype)) if subtype.subtypes(supertype) => (),
            (Some(_), Some(_)) => return false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Stm {
    Unit,
    Void,
}

impl Stm {
    pub fn least_upper_bound(&self, other: &Stm) -> Stm {
        match (self, other) {
            (Stm::Void, Stm::Void) => Stm::Void,
            _ => Stm::Unit,
        }
    }
}
