#[derive(Clone, Debug)]
pub enum Error {
    InvalidInteger,
    InvalidCharacter,
    InvalidEscape,
    UnknownCharacter,
    UnclosedCharacter,
    UnclosedString,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        | Error::InvalidInteger => write!(fmt, "invalid integer literal"),
        | Error::InvalidCharacter => write!(fmt, "invalid character literal"),
        | Error::InvalidEscape => write!(fmt, "invalid escape sequence"),
        | Error::UnclosedCharacter => write!(fmt, "unclosed character literal"),
        | Error::UnknownCharacter => write!(fmt, "unknown character"),
        | Error::UnclosedString => write!(fmt, "unclosed string literal"),
        }
    }
}
