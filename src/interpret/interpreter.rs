use std::io::{BufRead, BufReader, Stdin};
use std::collections::HashMap;

use crate::data::operand;
use crate::data::lir;
use crate::util::symbol;

#[derive(Debug)]
pub struct Interpreter<'unit> {
    /// Call stack
    call: Vec<Frame>, 

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
}

impl<'unit> Interpreter<'unit> {
    fn interpret_stm(&mut self) {
        unimplemented!()
    }

    fn interpret_exp(&self) -> Val {
        unimplemented!()
    }
}

#[derive(Debug)]
enum Val {
    Name(operand::Label),
    Temp(operand::Temp),
    Int(i64),
}

#[derive(Debug)]
struct Frame {
    rets: Vec<i64>,
    regs: HashMap<operand::Temp, i64>,
}

impl Frame {
    fn get_reg(&self, temp: operand::Temp) -> Option<i64> {
        self.regs.get(&temp).cloned()
    }

    fn get_ret(&self, ret: usize) -> Option<i64> {
        self.rets.get(ret).cloned() 
    }
}
