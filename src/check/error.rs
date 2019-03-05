use crate::util::span;
use crate::data::typ;

#[derive(Clone, Debug)]
pub struct Error {
    span: span::Span,
    kind: ErrorKind,
}

impl Error {
    pub fn new(span: span::Span, kind: ErrorKind) -> Self {
        Error { span, kind }
    }
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnboundVar,
    UnboundFun,
    NotVar,
    NotFun,
    NotExp,
    IndexEmpty,
    CallLength,
    WrongInit,
    WrongReturn,
    InitLength,
    Unreachable,
    Mismatch {
        expected: typ::Typ,
        found: typ::Typ,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}
