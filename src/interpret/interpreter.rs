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
    heap: interpret::Heap,

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
            let cond = self.interpret_exp(exp)?.extract_bool()?;
            let branch = if cond {
                self.jump.get(t)
                    .cloned()
                    .ok_or_else(|| interpret::Error::UnboundLabel(*t))?
            } else {
                self.jump.get(f)
                    .cloned()
                    .unwrap_or(self.next.unwrap() + 1)
            };
            self.next = Some(branch);
        }
        | Call(f, args) => unimplemented!(),
        | Label(exp) => (),
        | Move(lir::Exp::Mem(mem), src) => {
            let frame = self.call.current();
            let address = self.interpret_exp(mem)?.extract_int(frame)?;
            let value = self.interpret_exp(src)?.extract_int(frame)?;
            self.heap.store(address, value);
        }
        | Move(dst, src) => {
            let frame = self.call.current_mut();
            let dst = self.interpret_exp(dst)?.extract_temp()?;
            let src = self.interpret_exp(src)?.extract_int(frame)?;
            frame.insert(dst, src);
        }
        | Return(rets) => {
            let rets = rets.iter()
                .map(|ret| self.interpret_exp(ret))
                .map(|ret| ret.and_then(|ret| ret.extract_int(&self.call.current())))
                .collect::<Result<Vec<_>, _>>()?;

            let parent = self.call.parent_mut();

            for (i, ret) in rets.into_iter().enumerate() {
                parent.insert(operand::Temp::Ret(i), ret);
            }

            self.call.pop();
            self.next = None;
        },
        }
        Ok(())
    }

    fn interpret_exp(&self, exp: &lir::Exp) -> Result<interpret::Value, interpret::Error> {
        use lir::Exp::*;
        match exp {
        | Int(i) => Ok(interpret::Value::Int(*i)),
        | Name(l) => Ok(interpret::Value::Name(*l)),
        | Temp(t) => Ok(interpret::Value::Temp(*t)),
        | _ => unimplemented!(),
        }
    }
}
