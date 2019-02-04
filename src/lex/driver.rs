use std::io::{BufWriter, Write};

use crate::error;
use crate::lex;
use crate::util::Tap;

pub struct Driver<'main> {
    directory: &'main std::path::Path,
    diagnostic: bool,
}

impl<'main> Driver<'main> {
    pub fn new(directory: &'main std::path::Path, diagnostic: bool) -> Self {
        Driver { directory, diagnostic }
    }

    pub fn drive(&self, path: &std::path::Path) -> Result<Vec<lex::Spanned>, error::Error> {
        let source = std::fs::read_to_string(path)?;
        let lexer = lex::Lexer::new(&source);
        let mut tokens = Vec::new();
        if !self.diagnostic {
            for spanned in lexer {
                if spanned.is_err() {
                    tokens.push(spanned);
                    return Ok(tokens)
                } else {
                    tokens.push(spanned);
                }
            }
        } else {
            let mut log = self.directory
                .join(path)
                .with_extension("lexed")
                .tap(std::fs::File::create)
                .map(BufWriter::new)?;

            for spanned in lexer {
                match &spanned {
                | Ok((l, t, _)) => {
                    writeln!(&mut log, "{} {}", l, t)?;
                    tokens.push(spanned);
                }
                | Err(error) => {
                    writeln!(&mut log, "{}", error)?;
                    tokens.push(spanned);
                    return Ok(tokens)
                }
                }
            }
        }
        Ok(tokens)
    }
}
