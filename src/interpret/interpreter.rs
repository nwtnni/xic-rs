use std::io::{BufRead, BufReader, Stdin};
use std::collections::HashMap;

use crate::data::operand;
use crate::data::ir;
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

    fn interpret_call(&mut self, f: operand::Label, args: &[lir::Exp]) -> Result<(), interpret::Error> {
        self.call.push();

        // Push arguments into callee stack frame
        for (i, arg) in args.iter().enumerate() {
            let arg = self.interpret_exp(arg)?
                .extract_int(self.call.parent())?;
            self.call.current_mut()
                .insert(operand::Temp::Arg(i), arg);
        }

        // Record instruction pointer before jumping
        let next = self.next.unwrap() + 1;
        let jump = self.jump.get(&f)
            .cloned()
            .ok_or_else(|| interpret::Error::UnboundLabel(f))?;
        self.next = Some(jump);

        // Execute callee
        while let Some(next) = self.next {
            let stm = self.funs.get(next).ok_or_else(|| interpret::Error::InvalidIP)?;
            self.interpret_stm(stm)?;
        }

        // Pop callee stack frame and return instruction pointer
        self.call.pop();
        self.next = Some(next);
        Ok(())
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
        | Call(f, args) => {
            let f = self.interpret_exp(f)?
                .extract_name()?;
            self.interpret_call(f, args)?;
        }
        | Label(_) => (),
        | Move(lir::Exp::Mem(mem), src) => {
            let frame = self.call.current();
            let address = self.interpret_exp(mem)?.extract_int(frame)?;
            let value = self.interpret_exp(src)?.extract_int(frame)?;
            self.heap.store(address, value)?;
        }
        | Move(dst, src) => {
            let dst = self.interpret_exp(dst)?.extract_temp()?;
            let src = self.interpret_exp(src)?.extract_int(self.call.current())?;
            self.call.current_mut().insert(dst, src);
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
        | Mem(a) => {
            let frame = self.call.current();
            let address = self.interpret_exp(a)?.extract_int(frame)?;
            let value = self.heap.read(address)?;
            Ok(interpret::Value::Int(value))
        }
        | Bin(bin, l, r) => {
            let frame = self.call.current();
            let l = self.interpret_exp(l)?.extract_int(frame)?;
            let r = self.interpret_exp(r)?.extract_int(frame)?;
            use ir::Bin::*;
            let value = match bin {
            | Add => l + r,
            | Sub => l - r,
            | Mul => (l as i128 * r as i128) as i64,
            | Hul => ((l as i128 * r as i128) >> 64) as i64,
            | Div if r == 0 => return Err(interpret::Error::DivideByZero),
            | Mod if r == 0 => return Err(interpret::Error::DivideByZero),
            | Div => l / r,
            | Mod => l % r,
            | Xor => l ^ r,
            | Ls  => l << r,
            | Rs  => ((l as u64 >> r) as i64),
            | ARs => l >> r,
            | Lt  => if l < r  { 1 } else { 0 },
            | Le  => if l <= r { 1 } else { 0 },
            | Ge  => if l >= r { 1 } else { 0 },
            | Gt  => if l > r  { 1 } else { 0 },
            | Ne  => if l != r { 1 } else { 0 },
            | Eq  => if l == r { 1 } else { 0 },
            | And => if (l & r) & 0b1 == 1 { 1 } else { 0 },
            | Or  => if (l | r) & 0b1 == 1 { 1 } else { 0 },
            };
            Ok(interpret::Value::Int(value))
        }
        }
    }
}
