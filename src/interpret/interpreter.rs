use std::io::{BufRead, BufReader, Stdin};
use std::collections::HashMap;

use crate::data::operand;
use crate::data::lir;
use crate::util::symbol;
use crate::interpret;

#[derive(Debug)]
pub struct Interpreter<'unit> {
    /// Static data segment
    data: HashMap<operand::Label, symbol::Symbol>,

    /// Jump table
    jump: HashMap<operand::Label, usize>,

    /// Flattened IR unit instructions
    funs: Vec<&'unit lir::Stm>,

    /// Heap for dynamic memory allocation
    heap: Vec<i64>,

    /// `stdin` connection for I/O library functions
    read: BufReader<Stdin>,

    /// Call stack
    call: interpret::Stack,

    /// Instruction pointer
    next: usize
}

impl<'unit> Interpreter<'unit> {

    fn interpret_call(&mut self, fun: operand::Label) -> Result<Vec<i64>, interpret::Error> {
        unimplemented!()
    }

    fn interpret_stm(&mut self) -> Option<Vec<i64>> {
        unimplemented!()
    }

    fn interpret_exp(&self) {
        unimplemented!()
    }
}
