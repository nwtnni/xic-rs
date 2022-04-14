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
use crate::util::Tap as _;

pub fn lex(path: &Path, diagnostic: Option<&Path>) -> Result<token::Tokens, crate::Error> {
    let source = std::fs::read_to_string(path)?;
    let tokens = Lexer::new(&source).lex();

    if let Some(directory) = diagnostic {
        let mut file = directory
            .join(path)
            .with_extension("lexed")
            .tap(fs::File::create)
            .map(io::BufWriter::new)?;

        writeln!(&mut file, "{}", tokens)?;
    }

    Ok(tokens)
}
