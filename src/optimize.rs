mod constant_fold;
mod constant_propagate;
mod copy;
mod dead_code;
mod inline;
mod partial_redundancy;

pub use constant_fold::fold as constant_fold;
pub use constant_propagate::propagate as constant_propagate;
pub use copy::propagate as copy_propagate;
pub use dead_code::eliminate as eliminate_dead_code;
pub use inline::inline;
pub use partial_redundancy::eliminate as eliminate_partial_redundancy;
