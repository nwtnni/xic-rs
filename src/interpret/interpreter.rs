use std::io::Read;
use std::str::FromStr;
use std::collections::HashMap;

use crate::constants;
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

    /// Call stack
    call: interpret::Stack,

    /// Instruction pointer
    next: Option<usize>
}

impl<'unit> Interpreter<'unit> {
    pub fn new(unit: &'unit ir::Unit<lir::Fun>) -> Self {
        let mut funs = Vec::new();
        let mut jump = HashMap::default();
        let data = unit.data.iter()
            .map(|(sym, label)| (*label, *sym))
            .collect();

        for fun in unit.funs.values() {
            jump.insert(operand::Label::Fix(fun.name), funs.len());
            for stm in &fun.body {
                if let lir::Stm::Label(label) = stm {
                    jump.insert(*label, funs.len());
                }
                funs.push(stm);
            }
        }

        Interpreter {
            data, 
            jump,
            funs,
            heap: interpret::Heap::new(),
            call: interpret::Stack::new(),
            next: None,        
        }
    }

    pub fn interpret_unit(mut self) -> Result<(), interpret::Error> {
        self.interpret_call(
            operand::Label::Fix(symbol::intern("_Imain_paai")),
            &vec![lir::Exp::Int(0)],
        )?;
        Ok(())
    }

    fn interpret_call(&mut self, f: operand::Label, args: &[lir::Exp]) -> Result<(), interpret::Error> {

        // Try to interpret as a library function first
        if let operand::Label::Fix(f) = f {
            if let Ok(()) = self.interpret_lib(f, args) {
                return Ok(())
            }
        }

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
            .ok_or_else(|| interpret::Error::UnboundFun(f))?;
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

    fn interpret_lib(&mut self, f: symbol::Symbol, args: &[lir::Exp]) -> Result<(), interpret::Error> {
        if args.len() != match symbol::resolve(f) {
        | constants::GET_CHAR
        | constants::EOF
        | constants::XI_OUT_OF_BOUNDS => 0,
        | constants::PRINT
        | constants::PRINTLN
        | constants::READLN
        | constants::UNPARSE_INT
        | constants::PARSE_INT
        | constants::XI_ALLOC
        | constants::ASSERT => 1,
        | _ => return Err(interpret::Error::UnboundFun(operand::Label::Fix(f))),
        } {
            return Err(interpret::Error::CallMismatch)
        }

        match symbol::resolve(f) {
        | constants::PRINT | constants::PRINTLN => {
            let ptr = self.interpret_exp(&args[0])?.extract_int(self.call.current())?;
            let arr = self.heap.read_arr(ptr)?;
            for n in arr {
                let c = std::char::from_u32(n as u32)
                    .ok_or_else(|| interpret::Error::InvalidChar(n))?;
                print!("{}", c);
            }
            if symbol::resolve(f) == constants::PRINTLN {
                println!()
            }
        }
        | constants::READLN => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .unwrap();
            let len = buffer.len() as i64;
            let ptr = self.heap.malloc((len + 1) * constants::WORD_SIZE)?;
            self.heap.store_str(ptr, &buffer)?;
            self.interpret_ret(0, ptr + constants::WORD_SIZE);
        }
        | constants::GET_CHAR => {
            let mut buffer = [0u8];
            std::io::stdin()
                .read_exact(&mut buffer)
                .unwrap();
            self.interpret_ret(0, buffer[0] as i64);
        }
        | constants::EOF => {
            let eof = std::io::stdin()
                .bytes()
                .peekable()
                .peek()
                .is_none();
            self.interpret_ret(0, if eof { 1 } else { 0 });
        }
        | constants::UNPARSE_INT => {
            let s = self.interpret_exp(&args[0])?
                .extract_int(self.call.current())?
                .to_string();
            let len = s.len() as i64;
            let ptr = self.heap.malloc((len + 1) * constants::WORD_SIZE)?;
            self.heap.store_str(ptr, &s)?;
            self.interpret_ret(0, ptr + constants::WORD_SIZE);
        }
        | constants::PARSE_INT => {
            let ptr = self.interpret_exp(&args[0])?.extract_int(self.call.current())?;
            let arr = self.heap.read_arr(ptr)?;
            let mut string = String::new();
            for n in arr {
                let c = std::char::from_u32(n as u32)
                    .ok_or_else(|| interpret::Error::InvalidChar(n))?;
                string.push(c);
            }
            let (result, success) = match i64::from_str(&string) {
            | Ok(n) => (n, 1),
            | Err(_) => (0, 0),
            };
            self.interpret_ret(0, result);
            self.interpret_ret(1, success);
        }
        | constants::XI_ALLOC => {
            let bytes = self.interpret_exp(&args[0])?.extract_int(self.call.current())?;
            let ptr = self.heap.calloc(bytes)?;
            self.interpret_ret(0, ptr);
        }
        | constants::XI_OUT_OF_BOUNDS => {
            return Err(interpret::Error::OutOfBounds)
        }
        | constants::ASSERT => {
            let test = self.interpret_exp(&args[0])?.extract_bool(self.call.current())?;
            if !test { return Err(interpret::Error::AssertFail) }
        }
        | _ => unreachable!(),
        }
        Ok(())
    }

    fn interpret_stm(&mut self, stm: &lir::Stm) -> Result<(), interpret::Error> {
        use lir::Stm::*;
        match stm {
        | Label(_) => (),
        | Jump(exp) => {
            let label = self.interpret_exp(exp)?
                .extract_name()?;
            let next = self.jump.get(&label)
                .cloned()
                .ok_or_else(|| interpret::Error::UnboundLabel(label))?;
            self.next = Some(next);
        }
        | CJump(exp, t, f) => {
            let cond = self.interpret_exp(exp)?.extract_bool(self.call.current())?;
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

            for (i, ret) in rets.into_iter().enumerate() {
                self.interpret_ret(i, ret);
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

    fn interpret_ret(&mut self, index: usize, value: i64) {
        self.call
            .parent_mut()
            .insert(operand::Temp::Ret(index), value);
    }
}
