use crate::data::span;
use crate::error;

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
    InvalidCharacter,
    InvalidEscape,
    InvalidString,
    UnknownCharacter,
    UnclosedCharacter,
    UnclosedString,
}

impl ErrorKind {
    fn message(&self) -> &'static str {
        match self {
            ErrorKind::InvalidCharacter => "Invalid character literal",
            ErrorKind::InvalidEscape => "Invalid escape sequence",
            ErrorKind::InvalidString => "Invalid string literal",
            ErrorKind::UnclosedCharacter => "Unclosed character literal",
            ErrorKind::UnknownCharacter => "Unknown character",
            ErrorKind::UnclosedString => "Unclosed string literal",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} error:{}", self.span, self.kind.message())
    }
}

impl error::Report for Error {
    fn report(&self) -> ariadne::Report<span::Span> {
        use ariadne::Span as _;
        ariadne::Report::build(
            ariadne::ReportKind::Error,
            *self.span.source(),
            self.span.lo.idx,
        )
        .with_label(ariadne::Label::new(self.span).with_message(self.kind.message()))
        .finish()
    }
}
