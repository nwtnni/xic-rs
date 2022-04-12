mod global;
pub mod hir;
pub mod lir;
mod local;
mod postorder;

pub(crate) use global::Global;
pub(crate) use local::Local;
pub(crate) use postorder::Postorder;

use crate::data::operand;

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Integer(i64),
    Label(operand::Label),
    Memory(i64),
    Temporary(operand::Temporary),
}
