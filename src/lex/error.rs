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
        let description = match self.kind {
        | InvalidInteger    => "Invalid integer literal",
        | InvalidCharacter  => "Invalid character literal",
        | InvalidEscape     => "Invalid escape sequence",
        | UnclosedCharacter => "Unclosed character literal",
        | UnknownCharacter  => "Unknown character",
        | UnclosedString    => "Unclosed string literal",
        };
        write!(fmt, "{} error:{}", self.span, description)
    }
}
