use std::collections::HashMap;

use crate::data::operand;
use crate::data::ir;
use crate::data::lir;

#[derive(Debug)]
pub struct Interpreter<'unit> {
    unit: &'unit ir::Unit<lir::Fun>,
    heap: Vec<u8>,
    jump: HashMap<operand::Label, usize>,
}


