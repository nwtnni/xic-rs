mod conditional_propagate;
mod fold;
mod propagate;

pub use conditional_propagate::conditional_propagate_lir;
pub use fold::fold;
pub(crate) use fold::fold_binary;
pub(crate) use fold::fold_condition;
pub use propagate::propagate_assembly;
