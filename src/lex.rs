mod error;
mod lexer;

use std::path::Path;

pub(crate) use error::Error;
pub(crate) use error::ErrorKind;
pub(crate) use lexer::Lexer;

use crate::data::token;

pub fn lex(path: &Path) -> Result<token::Tokens, crate::Error> {
    let source = std::fs::read_to_string(path)?;
    let tokens = Lexer::new(&source).lex();
    Ok(tokens)
}
