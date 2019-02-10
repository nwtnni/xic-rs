use crate::error;
use crate::span;
use crate::token;

type ParseError = lalrpop_util::ParseError<span::Point, token::Token, error::Error>;

#[derive(Debug)]
pub struct Error {
    span: span::Span,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} error:Unexpected token", self.span)
    } 
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        unimplemented!()
    }
}
