use std::collections::BTreeMap;

use crate::data::ir;
use crate::data::operand;
use crate::interpret::Global;
use crate::interpret::Operand;
use crate::interpret::Postorder;
use crate::interpret::Value;
use crate::util::symbol::Symbol;

pub struct Local<'a, T: 'a> {
    postorder: &'a Postorder<T>,
    index: usize,
    temporaries: BTreeMap<operand::Temporary, Value>,
    stack: Vec<Operand>,
}

impl<'a, T: 'a> Local<'a, T> {
    pub fn new(unit: &'a ir::Unit<Postorder<T>>, name: &Symbol, arguments: &[Value]) -> Self {
        let postorder = unit.functions.get(name).unwrap();

        let mut temporaries = BTreeMap::new();

        for (index, argument) in arguments.iter().copied().enumerate() {
            temporaries.insert(operand::Temporary::Argument(index), argument);
        }

        Local {
            postorder,
            index: 0,
            temporaries,
            stack: Vec::new(),
        }
    }

    pub fn step(&mut self) -> Option<&'a T> {
        let index = self.index;
        self.index += 1;
        self.postorder.get_instruction(index)
    }

    pub fn jump(&mut self, label: &operand::Label) {
        self.index = self.postorder.get_label(label).copied().unwrap();
    }

    pub fn insert(&mut self, temporary: operand::Temporary, value: Value) {
        self.temporaries.insert(temporary, value);
    }

    pub fn push(&mut self, value: Operand) {
        self.stack.push(value);
    }

    pub fn pop(&mut self, global: &Global) -> Value {
        match self.stack.pop() {
            None => panic!("empty stack"),
            Some(Operand::Integer(integer)) => Value::Integer(integer),
            Some(Operand::Memory(address)) => global.read(address),
            Some(Operand::Label(label, offset)) => Value::Label(label, offset),
            Some(Operand::Temporary(temporary)) => self.temporaries[&temporary],
        }
    }

    pub fn pop_name(&mut self, global: &Global) -> Symbol {
        match self.pop(global) {
            Value::Label(operand::Label::Fixed(name), 8) => name,
            Value::Label(operand::Label::Fixed(_), _) => panic!("calling label offset"),
            Value::Label(operand::Label::Fresh(_, _), _) => panic!("calling fresh function name"),
            Value::Integer(_) => panic!("calling integer"),
        }
    }

    pub fn pop_arguments(&mut self, global: &Global, len: usize) -> Vec<Value> {
        let mut arguments = (0..len).map(|_| self.pop(global)).collect::<Vec<_>>();
        arguments.reverse();
        arguments
    }

    pub fn interpret_binary(&mut self, global: &Global, binary: &ir::Binary) {
        let right = self.pop(global);
        let left = self.pop(global);

        use ir::Binary::*;

        let value = match (binary, left, right) {
            (Add, Value::Label(label, l), Value::Integer(r)) => Operand::Label(label, l + r),
            (Add, Value::Integer(l), Value::Label(label, r)) => Operand::Label(label, l + r),
            (Sub, Value::Label(label, l), Value::Integer(r)) => Operand::Label(label, l - r),

            (Add, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l.wrapping_add(r)),
            (Sub, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l.wrapping_sub(r)),
            (Mul, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l.wrapping_mul(r)),
            (Hul, Value::Integer(l), Value::Integer(r)) => {
                Operand::Integer((((l as i128) * (r as i128)) >> 64) as i64)
            }
            // TODO: handle divide by 0
            (Div, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l / r),
            (Mod, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l % r),
            (Xor, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l ^ r),
            (Ls, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l << r),
            (Rs, Value::Integer(l), Value::Integer(r)) => {
                Operand::Integer(((l as u64) >> r) as i64)
            }
            (ARs, Value::Integer(l), Value::Integer(r)) => Operand::Integer(l >> r),
            (Lt, Value::Integer(l), Value::Integer(r)) => Operand::Integer((l < r) as bool as i64),
            (Le, Value::Integer(l), Value::Integer(r)) => Operand::Integer((l <= r) as bool as i64),
            (Ge, Value::Integer(l), Value::Integer(r)) => Operand::Integer((l >= r) as bool as i64),
            (Gt, Value::Integer(l), Value::Integer(r)) => Operand::Integer((l > r) as bool as i64),
            (Ne, Value::Integer(l), Value::Integer(r)) => Operand::Integer((l != r) as bool as i64),
            (Eq, Value::Integer(l), Value::Integer(r)) => Operand::Integer((l == r) as bool as i64),
            (And, Value::Integer(l), Value::Integer(r)) => {
                debug_assert!(l == 0 || l == 1);
                debug_assert!(r == 0 || r == 1);
                Operand::Integer(l & r)
            }
            (Or, Value::Integer(l), Value::Integer(r)) => {
                debug_assert!(l == 0 || l == 1);
                debug_assert!(r == 0 || r == 1);
                Operand::Integer(l | r)
            }

            _ => unreachable!(),
        };

        self.push(value);
    }

    pub fn interpret_jump(&mut self, global: &Global) {
        let label = match self.pop(global) {
            Value::Label(label, 8) => label,
            Value::Label(_, _) => panic!("jumping to label offset"),
            Value::Integer(_) => panic!("jumping to integer"),
        };
        self.jump(&label);
    }

    pub fn interpret_cjump(
        &mut self,
        global: &Global,
        r#true: &operand::Label,
        r#false: &operand::Label,
    ) {
        let label = match self.pop(global) {
            Value::Integer(0) => r#false,
            Value::Integer(1) => r#true,
            Value::Integer(_) => panic!("cjump on non-boolean condition"),
            Value::Label(_, _) => panic!("cjump on label condition"),
        };
        self.jump(label);
    }

    pub fn interpret_move(&mut self, global: &mut Global) {
        let from = self.pop(global);
        let into = self.stack.pop().unwrap();
        match into {
            Operand::Integer(_) => panic!("writing into integer"),
            Operand::Memory(address) => global.write(address, from),
            Operand::Temporary(temporary) => self.insert(temporary, from),
            Operand::Label(_, _) => panic!("writing into label"),
        }
    }
}
