#[allow(clippy::module_inception)]
mod check;
mod context;
mod error;

pub use check::check;
pub(crate) use context::Context;
pub(crate) use context::Entry;
pub(crate) use error::Error;
pub(crate) use error::ErrorKind;
