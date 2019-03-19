use crate::interpret;
use crate::data::ir;
use crate::data::lir;
use crate::error;

pub struct Driver {
    interpret: bool,
}

impl Driver {
    pub fn new(interpret: bool) -> Self {
        Driver { interpret }
    }

    pub fn drive(&self, lir: &ir::Unit<lir::Fun>) -> Result<(), error::Error> {
        if self.interpret {
            interpret::Interpreter::new(lir).interpret_unit()?;
        }
        Ok(())
    }
}
