mod error;
#[allow(clippy::module_inception)]
mod lex;

pub(crate) use error::Error;
pub(crate) use error::ErrorKind;
pub use lex::lex;
