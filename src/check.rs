#[allow(clippy::module_inception)]
mod check;
mod context;
mod error;

pub(crate) use check::Checker;
pub(crate) use context::Context;
pub(crate) use context::Entry;
pub(crate) use error::Error;
pub(crate) use error::ErrorKind;

use std::path::Path;

use crate::data::ast;

pub fn check(library: &Path, program: &ast::Program) -> Result<Context, crate::Error> {
    Checker::new().check_program(library, program)
}
