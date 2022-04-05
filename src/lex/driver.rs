use std::io::{BufWriter, Write};

use crate::data::token;
use crate::error;
use crate::lex;
use crate::util::span;
use crate::util::{TakeUntil, Tap};

type Spanned = Result<(span::Point, token::Token, span::Point), error::Error>;

pub struct Driver<'main> {
    directory: &'main std::path::Path,
    diagnostic: bool,
}

impl<'main> Driver<'main> {
    pub fn new(directory: &'main std::path::Path, diagnostic: bool) -> Self {
        Driver {
            directory,
            diagnostic,
        }
    }

    fn format<W: Write>(spanned: &Spanned, mut writer: W) -> Result<(), error::Error> {
        match spanned {
            Ok((l, t, _)) => writeln!(writer, "{} {}", l, t)?,
            Err(error) => writeln!(writer, "{}", error)?,
        }
        Ok(())
    }

    pub fn drive(&self, path: &std::path::Path) -> Result<Vec<Spanned>, error::Error> {
        let source = std::fs::read_to_string(path)?;
        let lexer = lex::Lexer::new(&source);
        let tokens = lexer.take_until(Result::is_err).collect::<Vec<_>>();

        if self.diagnostic {
            let mut log = self
                .directory
                .join(path)
                .with_extension("lexed")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            for spanned in &tokens {
                Self::format(spanned, &mut log)?;
            }
        }

        Ok(tokens)
    }
}
