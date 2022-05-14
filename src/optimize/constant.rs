mod fold;
mod propagate;

pub use fold::fold;
pub(crate) use fold::fold_binary;
pub(crate) use fold::fold_condition;
pub use propagate::propagate_assembly;
