use crate::span;

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
    InvalidInteger,
    InvalidCharacter,
    InvalidEscape,
    UnknownCharacter,
    UnclosedCharacter,
    UnclosedString,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use ErrorKind::*;
        match self.kind {
        | InvalidInteger => write!(fmt, "invalid integer literal"),
        | InvalidCharacter => write!(fmt, "invalid character literal"),
        | InvalidEscape => write!(fmt, "invalid escape sequence"),
        | UnclosedCharacter => write!(fmt, "unclosed character literal"),
        | UnknownCharacter => write!(fmt, "unknown character"),
        | UnclosedString => write!(fmt, "unclosed string literal"),
        }
    }
}
