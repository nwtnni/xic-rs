#[derive(Clone, Debug)]
pub enum Error {
    InvalidInteger,
    InvalidCharacter,
    UnknownCharacter,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        | Error::InvalidInteger => write!(fmt, "invalid integer literal"),
        | Error::InvalidCharacter => write!(fmt, "invalid character literal"),
        | Error::UnknownCharacter => write!(fmt, "unknown character"),
        }
    }
}
