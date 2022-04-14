use std::borrow::Cow;
use std::fmt;

use pretty::Arena;
use pretty::DocAllocator;
use pretty::DocBuilder;

use crate::error;
use crate::data::symbol;
use crate::util::Tap;

#[derive(Clone, Debug)]
pub enum Sexp {
    Atom(Cow<'static, str>),
    List(Vec<Sexp>),
}

impl fmt::Display for Sexp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let arena = Arena::new();
        self.to_doc(&arena).render_fmt(80, fmt)
    }
}

impl Sexp {
    fn to_doc<'a, A>(&self, allocator: &'a A) -> DocBuilder<'a, A, ()>
    where
        A: DocAllocator<'a, ()>,
        A::Doc: Clone,
    {
        match self {
            Sexp::Atom(atom) => allocator.text(atom.clone()),
            Sexp::List(list) => allocator
                .intersperse(
                    list.iter().map(|sexp| sexp.to_doc(allocator).nest(4)),
                    allocator.line(),
                )
                .parens()
                .group(),
        }
    }

    pub fn write<W: std::io::Write>(
        &self,
        width: usize,
        writer: &mut W,
    ) -> Result<(), error::Error> {
        let arena = Arena::new();
        self.to_doc(&arena).render(width, writer)?;
        Ok(())
    }
}

pub trait Serialize: Sized {
    fn sexp(&self) -> Sexp;

    fn sexp_move(self) -> Sexp {
        self.sexp()
    }
}

impl Serialize for Sexp {
    fn sexp(&self) -> Sexp {
        self.clone()
    }

    fn sexp_move(self) -> Sexp {
        self
    }
}

impl Serialize for i64 {
    fn sexp(&self) -> Sexp {
        Sexp::Atom(Cow::from(self.to_string()))
    }
}

impl Serialize for symbol::Symbol {
    fn sexp(&self) -> Sexp {
        Sexp::Atom(Cow::from(symbol::resolve(*self)))
    }
}

impl Serialize for &'static str {
    fn sexp(&self) -> Sexp {
        Sexp::Atom(Cow::from(*self))
    }
}

impl Serialize for String {
    fn sexp(&self) -> Sexp {
        Sexp::Atom(Cow::from(self.clone()))
    }

    fn sexp_move(self) -> Sexp {
        Sexp::Atom(Cow::from(self))
    }
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

impl<const N: usize, T: Serialize> Serialize for [T; N] {
    fn sexp(&self) -> Sexp {
        self.iter()
            .map(Serialize::sexp)
            .collect::<Vec<_>>()
            .tap(Sexp::List)
    }

    fn sexp_move(self) -> Sexp {
        IntoIterator::into_iter(self)
            .map(Serialize::sexp_move)
            .collect::<Vec<_>>()
            .tap(Sexp::List)
    }
}
