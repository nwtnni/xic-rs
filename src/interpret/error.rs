use crate::interpret;
use crate::data::operand;

#[derive(Debug)]
pub enum Error {
    UnboundTemp(operand::Temp),
    UnboundLabel(operand::Label),
    UnboundFun(operand::Label),
    NotName(interpret::Value),
    NotTemp(interpret::Value),
    NotInt(interpret::Value),
    NotBool(interpret::Value),
    InvalidMalloc(i64),
    InvalidRead(i64),
    InvalidWrite(i64),
    InvalidChar(i64),
    InvalidIP,
    OutOfMemory,
    DivideByZero,
    CallMismatch,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}
