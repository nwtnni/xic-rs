use std::borrow::Cow;

use crate::symbol;
use crate::util::Tap;

#[derive(Clone, Debug)]
pub enum Sexp {
    Atom(Cow<'static, str>),
    List(Vec<Sexp>), 
}

pub trait Serialize: Sized {
    fn sexp(&self) -> Sexp;
    fn sexp_move(self) -> Sexp { self.sexp() }
}

impl Serialize for Sexp {
    fn sexp(&self) -> Sexp { self.clone() }
    fn sexp_move(self) -> Sexp { self }
}

impl Serialize for symbol::Symbol {
    fn sexp(&self) -> Sexp { Sexp::Atom(Cow::from(symbol::resolve(*self))) }
}

impl Serialize for &'static str {
    fn sexp(&self) -> Sexp { Sexp::Atom(Cow::from(*self)) }
}

impl Serialize for String {
    fn sexp(&self) -> Sexp { Sexp::Atom(Cow::from(self.clone())) }
    fn sexp_move(self) -> Sexp { Sexp::Atom(Cow::from(self)) }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn sexp(&self) -> Sexp {
        self.iter()
            .map(Serialize::sexp)
            .collect::<Vec<_>>()
            .tap(Sexp::List)
    }

    fn sexp_move(self) -> Sexp {
        self.into_iter()
            .map(Serialize::sexp_move)
            .collect::<Vec<_>>()
            .tap(Sexp::List)
    }
}
