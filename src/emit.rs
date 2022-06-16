mod canonize;
#[allow(clippy::module_inception)]
mod emit;
mod print;

pub use canonize::emit_lir;
pub use emit::emit_hir;
