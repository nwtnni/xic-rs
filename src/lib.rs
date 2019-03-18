#![feature(box_patterns, fnbox)]

#[macro_use]
extern crate maplit;

pub mod data;
pub mod lex;
pub mod parse;
pub mod check;
pub mod emit;
pub mod interpret;
mod constants;
mod error;
mod util;

pub use error::Error;
