pub mod data;

mod abi;
mod allocate;
mod analyze;
mod assemble;
mod cfg;
mod check;
mod emit;
mod error;
mod interpret;
mod lex;
mod optimize;
mod parse;
mod util;

pub use error::Error;

pub mod api {
    pub use crate::allocate::allocate_linear;
    pub use crate::allocate::allocate_trivial;
    pub use crate::assemble::tile;
    pub use crate::cfg::construct_cfg;
    pub use crate::cfg::destruct_cfg;
    pub use crate::check::check;
    pub use crate::emit::emit_hir;
    pub use crate::emit::emit_lir;
    pub use crate::interpret::interpret_hir;
    pub use crate::interpret::interpret_lir;
    pub use crate::lex::lex;
    pub use crate::optimize::constant_fold_hir;
    pub use crate::optimize::constant_fold_lir;
    pub use crate::parse::parse;

    pub mod analyze {
        pub use crate::analyze::analyze;
        pub use crate::analyze::display;
        pub use crate::analyze::Analysis;
        pub use crate::analyze::LiveRanges;
        pub use crate::analyze::LiveVariables;
        pub use crate::analyze::Solution;
    }
}
