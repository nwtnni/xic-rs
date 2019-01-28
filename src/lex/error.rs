#[derive(Clone, Debug)]
pub enum Error {
    InvalidCharacter,
    UnknownCharacter,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        | Error::InvalidCharacter => write!(fmt, "invalid character constant"),
        | Error::UnknownCharacter => write!(fmt, "unknown character"),
        }
    }
}
