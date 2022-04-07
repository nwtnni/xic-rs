#![allow(clippy::clone_on_copy)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::try_err)]
#![allow(clippy::just_underscores_and_digits)]

mod driver;
mod error;
mod parser;
mod printer;

pub use driver::Driver;
pub(crate) use error::Error;
pub(crate) use parser::InterfaceParser;
pub(crate) use parser::ProgramParser;
