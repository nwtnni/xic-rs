pub mod analyze;
pub mod data;
pub mod optimize;

mod abi;
mod allocate;
mod assemble;
mod cfg;
mod check;
mod emit;
mod error;
mod interpret;
mod lex;
mod parse;
mod util;

type Map<K, V> = indexmap::IndexMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
type Set<T> = indexmap::IndexSet<T, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

pub use abi::Abi;
pub use abi::FramePointer;
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
}
