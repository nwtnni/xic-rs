use crate::interpret;
use crate::data::operand;

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Name(operand::Label),
    Temp(operand::Temp),
    Int(i64),
}

impl Value {
    pub fn extract_name(&self) -> Result<operand::Label, interpret::Error> {
        match *self {
        | Value::Name(label) => Ok(label),
        | value => Err(interpret::Error::NotName(value)),
        }
    }

    pub fn extract_temp(&self) -> Result<operand::Temp, interpret::Error> {
        match *self {
        | Value::Temp(temp) => Ok(temp),
        | value => Err(interpret::Error::NotTemp(value)),
        }
    }

    pub fn extract_bool(&self, frame: &interpret::Frame) -> Result<bool, interpret::Error> {
        match self.extract_int(frame) {
        | Ok(0)     => Ok(false),
        | Ok(1)     => Ok(true),
        | Ok(value) => Err(interpret::Error::NotBool(Value::Int(value))),
        | Err(interpret::Error::NotInt(value)) => Err(interpret::Error::NotBool(value)),
        | Err(err) => Err(err),
        }
    }

    pub fn extract_int(&self, frame: &interpret::Frame) -> Result<i64, interpret::Error> {
        match *self {
        | Value::Int(i) => Ok(i),
        | Value::Temp(temp) => frame.get(&temp),
        | value => Err(interpret::Error::NotInt(value)),
        }
    }
}
