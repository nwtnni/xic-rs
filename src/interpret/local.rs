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
}
