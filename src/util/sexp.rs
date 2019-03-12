use std::borrow::Cow;

use pretty::{BoxDoc, Doc};

use crate::error;
use crate::util::symbol;
use crate::util::Tap;

#[derive(Clone, Debug)]
pub enum Sexp {
    Atom(Cow<'static, str>),
    List(Vec<Sexp>), 
}

impl Sexp {
    fn to_doc(&self) -> Doc<BoxDoc<()>> {
        match self {
        | Sexp::Atom(s) => Doc::as_string(s),
        | Sexp::List(l) => {
            Doc::text("(")
                .append(
                    Doc::intersperse(
                        l.iter().map(Sexp::to_doc),
                        Doc::space(),
                    )
                    .nest(1)
                    .group()
                )
                .append(Doc::text(")"))
        }
        }
    }

    pub fn write<W: std::io::Write>(&self, width: usize, writer: &mut W) -> Result<(), error::Error> {
        self.to_doc().render(width, writer)?;
        Ok(())
    }
}

pub trait Serialize: Sized {
    fn sexp(&self) -> Sexp;
    fn sexp_move(self) -> Sexp { self.sexp() }
}

impl Serialize for Sexp {
    fn sexp(&self) -> Sexp { self.clone() }
    fn sexp_move(self) -> Sexp { self }
}

impl Serialize for i64 {
    fn sexp(&self) -> Sexp { Sexp::Atom(Cow::from(self.to_string())) }
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
