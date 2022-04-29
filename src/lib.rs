mod abi;
mod assemble;
mod check;
mod constants;
pub mod data;
mod emit;
mod error;
mod flow;
mod interpret;
mod lex;
mod optimize;
mod parse;
mod util;

pub use error::Error;

pub mod api {
    pub use crate::check::check;
    pub use crate::emit::emit_hir;
    pub use crate::emit::emit_lir;
    pub use crate::flow::construct_control_flow;
    pub use crate::flow::destruct_control_flow;
    pub use crate::interpret::interpret_hir;
    pub use crate::interpret::interpret_lir;
    pub use crate::lex::lex;
    pub use crate::optimize::fold_hir;
    pub use crate::optimize::fold_lir;
    pub use crate::parse::parse;
}
