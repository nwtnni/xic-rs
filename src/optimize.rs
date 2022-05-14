mod constant;
mod copy;
mod dead_code;
mod function;
mod r#loop;
mod partial_redundancy;

pub use constant::fold as fold_constants;
pub(crate) use constant::fold_binary;
pub(crate) use constant::fold_condition;
pub use constant::propagate_assembly as propagate_constants_assembly;
pub use copy::propagate_assembly as propagate_copies_assembly;
pub use dead_code::eliminate_assembly as eliminate_dead_code_assembly;
pub use function::inline_lir as inline_functions_lir;
pub use partial_redundancy::eliminate_lir as eliminate_partial_redundancy_lir;
pub use r#loop::invert_ast as invert_loops_ast;
