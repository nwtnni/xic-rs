mod check;
mod constants;
pub mod data;
mod emit;
mod error;
mod interpret;
mod lex;
mod parse;
mod util;

pub use error::Error;

pub mod api {
    pub use crate::check::check;
    pub use crate::emit::emit_hir;
    pub use crate::emit::emit_lir;
    pub use crate::emit::fold_hir;
    pub use crate::emit::fold_lir;
    pub use crate::interpret::interpret_hir;
    pub use crate::interpret::interpret_lir;
    pub use crate::lex::lex;
    pub use crate::parse::parse;
}
