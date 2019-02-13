use crate::error;
use crate::span;
use crate::token;

type ParseError = lalrpop_util::ParseError<span::Point, token::Token, error::Error>;

#[derive(Debug)]
pub enum Error {
    EOF,
    Integer(span::Span),
    Token(span::Span, token::Token),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
        | Error::EOF                => write!(fmt, "error:Unexpected eof"),
        | Error::Integer(span)      => write!(fmt, "{} error:Invalid integer literal", span),
        | Error::Token(span, token) => write!(fmt, "{} error:Unexpected token {}", span, token),
        }
    } 
}

impl From<ParseError> for error::Error {
    fn from(error: ParseError) -> Self {
        use lalrpop_util::ParseError::*;
        match error {
        | InvalidToken { .. }
        | ExtraToken { .. } => unreachable!(),
        | User { error } => error,
        | UnrecognizedToken { token: None, .. } => {
            error::Error::Syntactic(Error::EOF)
        }
        | UnrecognizedToken { token: Some((start, token, end)), .. } => {
            error::Error::Syntactic(Error::Token(span::Span::new(start, end), token))
        }
        }
    }
}
