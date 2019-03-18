use crate::interpret;
use crate::data::operand;

#[derive(Debug)]
pub enum Error {
    UnboundTemp(operand::Temp),
    UnboundLabel(operand::Label),
    NotName(interpret::Value),
    NotTemp(interpret::Value),
    NotInt(interpret::Value),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}
