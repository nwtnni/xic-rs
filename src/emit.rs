#[allow(clippy::module_inception)]
mod emit;
mod library;
mod lower;
mod print;

pub use emit::emit_hir;
pub use lower::emit_lir;
