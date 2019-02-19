#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Exp {
    Int,
    Bool,
    Arr(Box<Exp>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Stm {
    Unit,
    Void,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Typ {
    Exp(Exp),
    Tuple(Vec<Exp>),
    Unit,
}
