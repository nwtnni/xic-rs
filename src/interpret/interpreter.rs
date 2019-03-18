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
    next: Option<usize>
}

impl<'unit> Interpreter<'unit> {
    fn interpret_unit(mut self) {
        unimplemented!()
    }

    fn interpret_stm(&mut self, stm: &lir::Stm) -> Result<(), interpret::Error> {
        use lir::Stm::*;
        match stm {
        | Jump(exp) => {
            let label = self.interpret_exp(exp)?
                .extract_name()?;
            let next = self.jump.get(&label)
                .cloned()
                .ok_or_else(|| interpret::Error::UnboundLabel(label))?;
            self.next = Some(next);
        }
        | CJump(exp, t, f) => {
            let cond = self.interpret_exp(exp)?
                .extract_bool()?;
            let branch = if cond { t } else { f };
            let next = self.jump.get(branch)
                .cloned()
                .ok_or_else(|| interpret::Error::UnboundLabel(*branch))?;
            self.next = Some(next);
        }
        | Call(f, args) => unimplemented!(),
        | Label(exp) => (),
        | Move(lir::Exp::Mem(mem), src) => {
            unimplemented!()
        }
        | Move(dst, src) => {
            unimplemented!()
        }
        | Return(rets) => unimplemented!(),
        }
        Ok(())
    }

    fn interpret_exp(&self, exp: &lir::Exp) -> Result<interpret::Value, interpret::Error> {
        unimplemented!()
    }
}
