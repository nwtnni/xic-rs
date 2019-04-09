#![feature(box_patterns, fnbox)]

pub mod data;
pub mod lex;
pub mod parse;
pub mod check;
pub mod emit;
mod constants;
mod error;
mod util;

pub use error::Error;
