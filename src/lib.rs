mod check;
mod constants;
pub mod data;
pub mod emit;
mod error;
mod interpret;
mod lex;
mod parse;
mod util;

pub use error::Error;

pub mod api {
    pub use crate::check::check;
    pub use crate::lex::lex;
    pub use crate::parse::parse;
}
