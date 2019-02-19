#![feature(box_patterns)]

pub mod data;
pub mod lex;
pub mod parse;
pub mod check;
mod error;
mod util;

pub use error::Error;
