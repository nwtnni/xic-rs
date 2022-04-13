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

use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;

use crate::data::ast;
use crate::lex;
use crate::util::sexp::Serialize as _;
use crate::util::Tap as _;

pub fn parse<I>(
    path: &Path,
    diagnostic: Option<&Path>,
    tokens: I,
) -> Result<ast::Program, crate::Error>
where
    I: IntoIterator<Item = lex::Spanned>,
{
    let program = ProgramParser::new()
        .parse(tokens)
        .map_err(crate::Error::from);

    if let Some(directory) = diagnostic {
        let mut log = directory
            .join(path)
            .with_extension("parsed")
            .tap(fs::File::create)
            .map(io::BufWriter::new)?;

        match &program {
            Ok(program) => program.sexp().write(80, &mut log)?,
            Err(error) => write!(&mut log, "{}", error)?,
        };
    }

    program
}
