use crate::lex;
use crate::span;

#[derive(Clone, Debug)]
pub struct Error {
    span: span::Span,
    kind: Kind,
}

impl Error {
    pub fn lexical(span: span::Span, error: lex::Error) -> Self {
        Error { span, kind: Kind::Lexical(error) }
    }
}

#[derive(Clone, Debug)]
pub enum Kind {
    Lexical(lex::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.kind {
        | Kind::Lexical(error) => write!(fmt, "{}: {}", self.span.lo, error),
        }
    }
}

impl std::error::Error for Error {}
