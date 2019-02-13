use crate::error;
use crate::span;
use crate::token;

type ParseError = lalrpop_util::ParseError<span::Point, token::Token, error::Error>;

#[derive(Debug)]
pub struct Error {
    token: Option<(span::Span, token::Token)>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.token {
        | None => write!(fmt, "error:Unexpected eof"),
        | Some((span, token)) => write!(fmt, "{} error:Unexpected token {}", span, token),
        }
    } 
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        use lalrpop_util::ParseError::*;
        match error {
        | User { .. }
        | InvalidToken { .. }
        | ExtraToken { .. } => unreachable!(),
        | UnrecognizedToken { token: None, .. } => {
            Error { token: None }
        }
        | UnrecognizedToken { token: Some((start, token, end)), .. } => {
            Error { token: Some((span::Span::new(start, end), token)) }
        }
        }
    }
}
