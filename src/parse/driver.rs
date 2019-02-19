use std::io::{BufWriter, Write};

use crate::error;
use crate::parse;
use crate::data::ast;
use crate::data::token;
use crate::util::sexp::Serialize;
use crate::util::span;
use crate::util::{Conv, Tap};

pub struct Driver<'main> {
    directory: &'main std::path::Path,
    diagnostic: bool,
}

type Spanned = Result<(span::Point, token::Token, span::Point), error::Error>;

impl<'main> Driver<'main> {
    pub fn new(directory: &'main std::path::Path, diagnostic: bool) -> Self {
        Driver { directory, diagnostic }
    }

    pub fn drive<I>(&self, path: &std::path::Path, iter: I) -> Result<ast::Program, error::Error>
    where I: IntoIterator<Item = Spanned>
    {
        let program = parse::ProgramParser::new()
            .parse(iter)
            .map_err(Conv::conv::<error::Error>);

        if self.diagnostic {
            let mut log = self.directory
                .join(path)
                .with_extension("parsed")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            match &program {
            | Ok(program) => program.sexp().write(80, &mut log)?,
            | Err(error) => write!(&mut log, "{}", error)?,
            };
        }

        Ok(program?)
    }
}
