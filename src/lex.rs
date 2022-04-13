mod driver;
mod error;
mod lexer;

pub use driver::Driver;
pub(crate) use error::Error;
pub(crate) use error::ErrorKind;
pub(crate) use lexer::Lexer;

use crate::data::token;
use crate::util::span;

pub type Spanned = Result<(span::Point, token::Token, span::Point), crate::error::Error>;
