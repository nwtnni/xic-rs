#![allow(clippy::clone_on_copy)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::try_err)]
#![allow(clippy::just_underscores_and_digits)]

use lalrpop_util::lalrpop_mod;

mod driver;
mod error;
mod printer;
mod shim;

lalrpop_mod!(parser, "/parse/parser.rs");

pub use driver::Driver;
pub(crate) use error::Error;
pub(crate) use parser::{InterfaceParser, ProgramParser};
pub(crate) use shim::PreExp;
