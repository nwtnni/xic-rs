mod constant_fold;
mod copy;
mod dead_code;

pub use constant_fold::fold as constant_fold;
pub use copy::propagate as copy_propagate;
pub use dead_code::eliminate as eliminate_dead_code;
