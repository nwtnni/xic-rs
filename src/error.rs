use crate::check;
use crate::lex;
use crate::parse;

#[derive(Debug)]
pub enum Error {
    Lexical(lex::Error),
    Syntactic(parse::Error),
    Semantic(check::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Lexical(error) => write!(fmt, "{}", error),
            Error::Syntactic(error) => write!(fmt, "{}", error),
            Error::Semantic(error) => write!(fmt, "{}", error),
            Error::Io(error) => write!(fmt, "{}", error),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<lex::Error> for Error {
    fn from(error: lex::Error) -> Self {
        Error::Lexical(error)
    }
}

impl From<parse::Error> for Error {
    fn from(error: parse::Error) -> Self {
        Error::Syntactic(error)
    }
}

impl From<check::Error> for Error {
    fn from(error: check::Error) -> Self {
        Error::Semantic(error)
    }
}
