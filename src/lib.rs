mod abi;
mod assemble;
mod cfg;
mod check;
pub mod data;
mod emit;
mod error;
mod interpret;
mod lex;
mod optimize;
mod parse;
mod util;

pub use error::Error;

pub mod api {
    pub use crate::assemble::allocate;
    pub use crate::assemble::tile;
    pub use crate::cfg::construct_control_flow_assembly;
    pub use crate::cfg::construct_control_flow_lir;
    pub use crate::cfg::destruct_control_flow_assembly;
    pub use crate::cfg::destruct_control_flow_lir;
    pub use crate::check::check;
    pub use crate::emit::emit_hir;
    pub use crate::emit::emit_lir;
    pub use crate::interpret::interpret_hir;
    pub use crate::interpret::interpret_lir;
    pub use crate::lex::lex;
    pub use crate::optimize::fold_hir;
    pub use crate::optimize::fold_lir;
    pub use crate::parse::parse;
}
