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
    pub use crate::cfg::clean_cfg;
    pub use crate::cfg::construct_cfg;
    pub use crate::cfg::destruct_cfg;
    pub use crate::check::check;
    pub use crate::emit::emit_hir;
    pub use crate::emit::emit_lir;
    pub use crate::interpret::interpret_hir;
    pub use crate::interpret::interpret_lir;
    pub use crate::lex::lex;
    pub use crate::parse::parse;

    pub mod optimize {
        pub use crate::optimize::constant_fold;
        pub use crate::optimize::constant_propagate;
        pub use crate::optimize::copy_propagate;
        pub use crate::optimize::eliminate_dead_code;
    }

    pub mod analyze {
        pub use crate::analyze::analyze;
        pub use crate::analyze::display;
        pub use crate::analyze::Analysis;
        pub use crate::analyze::ConstantPropagation;
        pub use crate::analyze::CopyPropagation;
        pub use crate::analyze::LiveRanges;
        pub use crate::analyze::LiveVariables;
        pub use crate::analyze::Solution;
    }
}
