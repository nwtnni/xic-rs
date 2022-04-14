mod global;
mod hir;
mod lir;
mod local;
mod postorder;

pub use hir::interpret_hir;
pub use lir::interpret_lir;

pub(crate) use global::Global;
pub(crate) use local::Local;
pub(crate) use postorder::Postorder;

use crate::data::operand;

#[derive(Copy, Clone, Debug)]
pub enum Operand {
    Integer(i64),
    Label(operand::Label, i64),
    Memory(Value),
    Temporary(operand::Temporary),
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Integer(i64),
    Label(operand::Label, i64),
}

impl Value {
    #[track_caller]
    pub fn into_integer(self) -> i64 {
        match self {
            Value::Integer(integer) => integer,
            Value::Label(label, offset) => panic!(
                "Expected integer, but found label: [{:?} + {}]",
                label, offset
            ),
        }
    }

    pub fn into_operand(self) -> Operand {
        match self {
            Value::Integer(integer) => Operand::Integer(integer),
            Value::Label(label, offset) => Operand::Label(label, offset),
        }
    }
}
