#![feature(box_patterns, fnbox)]

pub mod check;
mod constants;
pub mod data;
pub mod emit;
mod error;
pub mod lex;
pub mod parse;
mod util;

pub use error::Error;
