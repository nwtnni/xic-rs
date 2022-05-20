use crate::data::span;
use crate::data::token;
use crate::error;

type ParseError = lalrpop_util::ParseError<span::Point, token::Token, error::Error>;

#[derive(Debug)]
pub enum Error {
    Eof(span::Point),
    Integer(span::Span),
    Array(span::Span),
    Length(span::Span),
    Token(span::Span, token::Token),
}

impl Error {
    fn message(&self) -> &'static str {
        match self {
            Error::Eof(_) => "Unexpected EOF",
            Error::Integer(_) => "Invalid integer literal",
            Error::Array(_) => "Unexpected length in array type",
            Error::Length(_) => "Declared array length after undeclared array length",
            Error::Token(_, _) => "Unexpected token",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message = self.message();
        match &self {
            Error::Eof(point) => write!(fmt, "{} error:{}", point, message),
            Error::Integer(span) | Error::Array(span) | Error::Length(span) => {
                write!(fmt, "{} error:{}", span, message)
            }
            Error::Token(span, token) => write!(fmt, "{} error:{} {}", span, message, token),
        }
    }
}

impl From<Error> for ParseError {
    fn from(error: Error) -> ParseError {
        lalrpop_util::ParseError::User {
            error: error.into(),
        }
    }
}

impl From<ParseError> for error::Error {
    fn from(error: ParseError) -> Self {
        use lalrpop_util::ParseError::*;
        match error {
            InvalidToken { .. } | ExtraToken { .. } => unreachable!(),
            User { error } => error,
            UnrecognizedEOF { location, .. } => error::Error::Syntactic(Error::Eof(location)),
            UnrecognizedToken {
                token: (start, token, end),
                ..
            } => error::Error::Syntactic(Error::Token(span::Span::new(start, end), token)),
        }
    }
}

impl error::Report for Error {
    fn report(&self) -> ariadne::Report<span::Span> {
        use ariadne::Span as _;

        const ERROR: ariadne::ReportKind = ariadne::ReportKind::Error;

        let message = self.message();

        match self {
            Error::Eof(point) => ariadne::Report::build(ERROR, point.path.unwrap(), point.idx)
                .with_message(message)
                .finish(),
            Error::Integer(span)
            | Error::Array(span)
            | Error::Length(span)
            | Error::Token(span, _) => ariadne::Report::build(ERROR, *span.source(), span.lo.idx)
                .with_label(ariadne::Label::new(*span).with_message(message))
                .finish(),
        }
    }
}
