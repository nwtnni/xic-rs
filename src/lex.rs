mod error;
mod lexer;

use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;

pub(crate) use error::Error;
pub(crate) use error::ErrorKind;
pub(crate) use lexer::Lexer;

use crate::data::token;
use crate::util::span;
use crate::util::TakeUntil as _;
use crate::util::Tap as _;

pub type Spanned = Result<(span::Point, token::Token, span::Point), crate::Error>;

pub fn lex(path: &Path, diagnostic: Option<&Path>) -> Result<Vec<Spanned>, crate::Error> {
    let source = std::fs::read_to_string(path)?;
    let lexer = Lexer::new(&source);
    let tokens = lexer.take_until(Result::is_err).collect::<Vec<_>>();

    if let Some(directory) = diagnostic {
        let mut file = directory
            .join(path)
            .with_extension("lexed")
            .tap(fs::File::create)
            .map(io::BufWriter::new)?;

        for spanned in &tokens {
            match spanned {
                Ok((left, token, _)) => writeln!(&mut file, "{} {}", left, token)?,
                Err(error) => writeln!(&mut file, "{}", error)?,
            }
        }
    }

    Ok(tokens)
}
