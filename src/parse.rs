#![allow(clippy::clone_on_copy)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::try_err)]
#![allow(clippy::just_underscores_and_digits)]

mod error;
mod parser;
mod printer;

pub(crate) use error::Error;
pub(crate) use parser::InterfaceParser;
pub(crate) use parser::ProgramParser;

use crate::data::ast;
use crate::data::token;

pub fn parse<I>(tokens: I) -> Result<ast::Program, crate::Error>
where
    I: IntoIterator<Item = token::Spanned>,
{
    ProgramParser::new()
        .parse(tokens)
        .map_err(crate::Error::from)
}
