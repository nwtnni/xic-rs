#![allow(clippy::clone_on_copy)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::try_err)]
#![allow(clippy::just_underscores_and_digits)]
#![allow(unused_imports)]

mod error;
mod grammar;
mod print;

pub(crate) use error::Error;
pub(crate) use grammar::InterfaceParser;
pub(crate) use grammar::ProgramParser;

use std::path::Path;

use crate::data::ast;
use crate::data::token;
use crate::util;

pub fn parse<I>(path: &Path, tokens: I) -> Result<ast::Program, crate::Error>
where
    I: IntoIterator<Item = token::Spanned>,
{
    log::info!(
        "[{}] Parsing {}...",
        std::any::type_name::<token::Tokens>(),
        path.display()
    );
    util::time!(
        "[{}] Done parsing {}",
        std::any::type_name::<token::Tokens>(),
        path.display()
    );

    ProgramParser::new()
        .parse(tokens)
        .map_err(crate::Error::from)
}
