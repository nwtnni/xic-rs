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
