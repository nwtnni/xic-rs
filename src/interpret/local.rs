use std::collections::BTreeMap;

use crate::data::ir;
use crate::data::operand;
use crate::interpret::global::Global;
use crate::interpret::global::Value;
use crate::interpret::postorder::Postorder;
use crate::util::symbol::Symbol;

pub struct Frame<'a, T: 'a> {
    postorder: &'a Postorder<T>,
    index: usize,
    temporaries: BTreeMap<operand::Temporary, i64>,
    stack: Vec<Value>,
}

impl<'a, T: 'a> Frame<'a, T> {
    pub fn new(unit: &'a ir::Unit<Postorder<T>>, name: &Symbol, arguments: &[i64]) -> Self {
        let postorder = unit.functions.get(name).unwrap();

        let mut temporaries = BTreeMap::new();

        for (index, argument) in arguments.iter().copied().enumerate() {
            temporaries.insert(operand::Temporary::Argument(index), argument);
        }

        Frame {
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

    pub fn insert(&mut self, temporary: operand::Temporary, value: i64) {
        self.temporaries.insert(temporary, value);
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn pop_arguments(&mut self, global: &Global, len: usize) -> Vec<i64> {
        let mut arguments = (0..len)
            .map(|_| self.pop_integer(global))
            .collect::<Vec<_>>();

        arguments.reverse();
        arguments
    }

    pub fn pop_integer(&mut self, global: &Global) -> i64 {
        match self.stack.pop() {
            None => panic!("empty stack"),
            Some(Value::Integer(integer)) => integer,
            Some(Value::Memory(address)) => global.read(address),
            Some(Value::Label(_)) => panic!("using label as integer"),
            Some(Value::Temporary(temporary)) => self.temporaries[&temporary],
        }
    }

    pub fn pop_label(&mut self) -> operand::Label {
        match self.stack.pop() {
            None => panic!("empty stack"),
            Some(Value::Integer(_)) => panic!("using integer as label"),
            Some(Value::Memory(_)) => panic!("using memory as label"),
            Some(Value::Label(label)) => label,
            Some(Value::Temporary(_)) => panic!("using temporary as label"),
        }
    }

    pub fn interpret_binary(&mut self, global: &Global, binary: &ir::Binary) {
        let right = self.pop_integer(global);
        let left = self.pop_integer(global);
        let value = match binary {
            ir::Binary::Add => left.wrapping_add(right),
            ir::Binary::Sub => left.wrapping_sub(right),
            ir::Binary::Mul => left.wrapping_mul(right),
            ir::Binary::Hul => (((left as i128) * (right as i128)) >> 64) as i64,
            ir::Binary::Div => left / right,
            ir::Binary::Mod => left % right,
            ir::Binary::Xor => left ^ right,
            ir::Binary::Ls => left << right,
            ir::Binary::Rs => ((left as u64) >> right) as i64,
            ir::Binary::ARs => left >> right,
            ir::Binary::Lt => (left < right) as bool as i64,
            ir::Binary::Le => (left <= right) as bool as i64,
            ir::Binary::Ge => (left >= right) as bool as i64,
            ir::Binary::Gt => (left > right) as bool as i64,
            ir::Binary::Ne => (left != right) as bool as i64,
            ir::Binary::Eq => (left == right) as bool as i64,
            ir::Binary::And => {
                debug_assert!(left == 0 || left == 1);
                debug_assert!(right == 0 || right == 1);
                left & right
            }
            ir::Binary::Or => {
                debug_assert!(left == 0 || left == 1);
                debug_assert!(right == 0 || right == 1);
                left | right
            }
        };
        self.push(Value::Integer(value));
    }

    pub fn interpret_jump(&mut self) {
        let label = self.pop_label();
        self.jump(&label);
    }

    pub fn interpret_cjump(
        &mut self,
        global: &Global,
        r#true: &operand::Label,
        r#false: &operand::Label,
    ) {
        let label = match self.pop_integer(global) {
            0 => r#false,
            1 => r#true,
            _ => unreachable!(),
        };
        self.jump(label);
    }

    pub fn interpret_move(&mut self, global: &mut Global) {
        let from = self.pop_integer(global);
        let into = self.pop();
        match into {
            Value::Integer(_) => panic!("writing into integer"),
            Value::Memory(address) => global.write(address, from),
            Value::Temporary(temporary) => self.insert(temporary, from),
            Value::Label(_) => panic!("writing into label"),
        }
    }
}
