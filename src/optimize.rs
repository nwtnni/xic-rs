mod constant_fold;
mod dead_code;

pub use constant_fold::fold as constant_fold;
pub use dead_code::eliminate as eliminate_dead_code;
