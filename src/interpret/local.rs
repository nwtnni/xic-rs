use crate::data::ir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol::Symbol;
use crate::interpret::Global;
use crate::interpret::Operand;
use crate::interpret::Postorder;
use crate::interpret::Value;
use crate::Map;

pub struct Local<'a, T: 'a> {
    postorder: &'a Postorder<T>,
    index: usize,
    temporaries: Map<Temporary, Value>,
    stack: Vec<Operand>,
}

impl<'a, T: 'a> Local<'a, T> {
    pub fn new(unit: &'a ir::Unit<Postorder<T>>, name: &Symbol, arguments: &[Value]) -> Self {
        Local {
            postorder: unit.functions.get(name).unwrap(),
            index: 0,
            temporaries: arguments
                .iter()
                .copied()
                .enumerate()
                .map(|(index, argument)| (Temporary::Argument(index), argument))
                .collect(),
            stack: Vec::new(),
        }
    }

    pub fn step(&mut self) -> Option<&'a T> {
        let index = self.index;
        self.index += 1;
        self.postorder.get_statement(index)
    }

    pub fn insert(&mut self, temporary: Temporary, value: Value) {
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
            Some(Operand::Temporary(temporary)) => self
                .temporaries
                .get(&temporary)
                .copied()
                .unwrap_or_else(|| panic!("unbound temporary: {}", temporary)),
        }
    }

    pub fn pop_name(&mut self, global: &Global) -> Symbol {
        match self.pop(global) {
            Value::Label(Label::Fixed(name), 0) => name,
            Value::Label(Label::Fixed(_), _) => panic!("calling label offset"),
            Value::Label(Label::Fresh(_, _), _) => panic!("calling fresh function name"),
            Value::Integer(_) => panic!("calling integer"),
        }
    }

    pub fn pop_list(&mut self, global: &Global, len: usize) -> Vec<Value> {
        let mut list = (0..len).map(|_| self.pop(global)).collect::<Vec<_>>();
        list.reverse();
        list
    }

    pub fn interpret_condition(&mut self, global: &Global, condition: &ir::Condition) -> bool {
        let right = self.pop(global);
        let left = self.pop(global);

        let (left, right) = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => (left, right),
            (Value::Label(left, left_offset), Value::Label(right, right_offset))
                if left == right =>
            {
                (left_offset, right_offset)
            }
            (_, _) => unreachable!(),
        };

        match condition {
            ir::Condition::Lt => left < right,
            ir::Condition::Le => left <= right,
            ir::Condition::Ge => left >= right,
            ir::Condition::Gt => left > right,
            ir::Condition::Ne => left != right,
            ir::Condition::Eq => left == right,
            ir::Condition::Ae => left as u64 >= right as u64,
        }
    }

    pub fn interpret_binary(&mut self, global: &Global, binary: &ir::Binary) {
        let right = self.pop(global);
        let left = self.pop(global);

        let (left, right) = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => (left, right),

            (Value::Label(label, left), Value::Integer(right))
            | (Value::Integer(right), Value::Label(label, left))
                if *binary == ir::Binary::Add =>
            {
                self.push(Operand::Label(label, left + right));
                return;
            }

            (Value::Label(label, left), Value::Integer(right)) if *binary == ir::Binary::Sub => {
                self.push(Operand::Label(label, left - right));
                return;
            }

            _ => unreachable!(),
        };

        let value = match binary {
            ir::Binary::Add => left.wrapping_add(right),
            ir::Binary::Sub => left.wrapping_sub(right),
            ir::Binary::Mul => left.wrapping_mul(right),
            ir::Binary::Hul => (((left as i128) * (right as i128)) >> 64) as i64,
            // TODO: handle divide by 0
            ir::Binary::Div => left / right,
            ir::Binary::Mod => left % right,
            ir::Binary::Xor => left ^ right,
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

        self.push(Operand::Integer(value));
    }

    pub fn interpret_jump(&mut self, label: &Label) {
        self.index = self.postorder.get_label(label).copied().unwrap();
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
