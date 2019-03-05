#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Exp {
    Int,
    Bool,
    Any,
    Arr(Box<Exp>),
}

impl Exp {
    pub fn subtypes(&self, rhs: &Exp) -> bool {
        match (self, rhs) {
        | (Exp::Any, _)
        | (Exp::Int, Exp::Int)
        | (Exp::Bool, Exp::Bool) => true,
        | (Exp::Arr(l), Exp::Arr(r)) => l.subtypes(r),
        | _ => false,
        }
    }
}

impl std::fmt::Display for Exp {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        | Exp::Int => write!(fmt, "int"),
        | Exp::Bool => write!(fmt, "bool"),
        | Exp::Any => write!(fmt, "any"), // Panic?
        | Exp::Arr(typ) => write!(fmt, "{}[]", typ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Stm {
    Unit,
    Void,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Typ {
    Exp(Exp),
    Tup(Vec<Exp>),
    Unit,
}

impl std::fmt::Display for Typ {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        | Typ::Unit => write!(fmt, "unit"),
        | Typ::Exp(typ) => write!(fmt, "{}", typ),
        | Typ::Tup(typs) => {
            write!(fmt, "(")?;
            if typs.len() > 0 {
                let mut iter = typs.iter();
                if let Some(typ) = iter.next() {
                    write!(fmt, "{}", typ)?;
                }
                for typ in iter {
                    write!(fmt, ", {}", typ)?;
                }
            }
            write!(fmt, ")")
        }
        }
    }
}

impl Typ {
    #[inline(always)]
    pub fn any() -> Self {
        Typ::Exp(Exp::Any)
    }

    #[inline(always)]
    pub fn int() -> Self {
        Typ::Exp(Exp::Int)
    }

    #[inline(always)]
    pub fn boolean() -> Self {
        Typ::Exp(Exp::Bool)
    }

    #[inline(always)]
    pub fn array(typ: Exp) -> Self {
        Typ::Exp(Exp::Arr(Box::new(typ)))
    }

    pub fn subtypes(&self, rhs: &Typ) -> bool {
        match (self, rhs) {
        | (Typ::Exp(l), Typ::Exp(r)) => l.subtypes(r),
        | (Typ::Tup(l), Typ::Tup(r)) if l.len() == r.len() => {
            l.iter().zip(r.iter()).all(|(l, r)| l.subtypes(r))
        }
        | _ => false,
        }
    }
}
