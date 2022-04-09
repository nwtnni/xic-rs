#![allow(dead_code)]

use std::collections::BTreeMap;
use std::convert::TryFrom as _;
use std::io::Read as _;

use rand::rngs::ThreadRng;
use rand::Rng as _;

use crate::constants;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand;
use crate::util::symbol;
use crate::util::symbol::Symbol;

const HEAP_SIZE: usize = 1024;

pub struct Global {
    heap: Vec<i64>,
    rng: ThreadRng,
}

struct Local<'a> {
    instructions: &'a Vec<Hir<'a>>,
    labels: &'a BTreeMap<operand::Label, usize>,
    index: usize,
    temporaries: BTreeMap<operand::Temporary, i64>,
    stack: Vec<Value>,
}

#[derive(Copy, Clone, Debug)]
enum Value {
    Integer(i64),
    Label(operand::Label),
    Memory(i64),
    Temporary(operand::Temporary),
}

impl Global {
    pub fn run(unit: &ir::Unit<hir::Function>) {
        let unit = unit.map(Flat::flatten_function);
        let mut global = Global {
            heap: Vec::new(),
            rng: rand::thread_rng(),
        };

        assert!(global
            .call(&unit, &symbol::intern("_Imain_paai"), &[0])
            .is_empty());
    }

    fn call(&mut self, unit: &ir::Unit<Flat>, name: &Symbol, arguments: &[i64]) -> Vec<i64> {
        let Flat {
            instructions,
            labels,
        } = unit.functions.get(name).unwrap();

        let mut temporaries = BTreeMap::new();

        for (index, argument) in arguments.iter().copied().enumerate() {
            temporaries.insert(operand::Temporary::Argument(index), argument);
        }

        let mut frame = Local {
            instructions,
            labels,
            index: 0,
            temporaries,
            stack: Vec::new(),
        };

        loop {
            match frame.step(unit, self) {
                Some(returns) => return returns,
                None => continue,
            }
        }
    }
}

impl<'a> Local<'a> {
    fn step(&mut self, unit: &ir::Unit<Flat>, global: &mut Global) -> Option<Vec<i64>> {
        let instruction = match self.instructions.get(self.index) {
            Some(instruction) => instruction,
            None => return Some(Vec::new()),
        };

        match instruction {
            Hir::Expression(hir::Expression::Integer(integer)) => {
                self.stack.push(Value::Integer(*integer));
            }
            Hir::Expression(hir::Expression::Label(label)) => {
                self.stack.push(Value::Label(*label));
            }
            Hir::Expression(hir::Expression::Temporary(temporary)) => {
                self.stack.push(Value::Temporary(*temporary));
            }
            Hir::Expression(hir::Expression::Memory(_)) => {
                let address = self.pop_integer(global);
                self.stack.push(Value::Memory(address));
            }
            Hir::Expression(hir::Expression::Binary(binary, _, _)) => {
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
                        assert!(left == 0 || left == 1);
                        assert!(right == 0 || right == 1);
                        left & right
                    }
                    ir::Binary::Or => {
                        assert!(left == 0 || left == 1);
                        assert!(right == 0 || right == 1);
                        left | right
                    }
                };
                self.stack.push(Value::Integer(value));
            }
            Hir::Expression(hir::Expression::Call(call)) => {
                let mut r#return = self.call(unit, global, call);
                assert_eq!(r#return.len(), 1);
                self.stack.push(Value::Integer(r#return.remove(0)));
            }
            Hir::Expression(hir::Expression::Sequence(_, _)) => unreachable!(),

            Hir::Statement(hir::Statement::Jump(_)) => {
                let label = self.pop_label();
                let index = self.labels.get(&label).unwrap();
                self.index = *index;
                return None;
            }
            Hir::Statement(hir::Statement::CJump(_, r#true, r#false)) => {
                let label = match self.pop_integer(global) {
                    0 => r#false,
                    1 => r#true,
                    _ => unreachable!(),
                };
                let index = self.labels.get(label).unwrap();
                self.index = *index;
                return None;
            }
            Hir::Statement(hir::Statement::Label(_)) => unreachable!(),
            Hir::Statement(hir::Statement::Call(call)) => {
                for (index, r#return) in self.call(unit, global, call).into_iter().enumerate() {
                    self.temporaries
                        .insert(operand::Temporary::Return(index), r#return);
                }
            }
            Hir::Statement(hir::Statement::Move(_, _)) => {
                let from = self.pop_integer(global);
                let into = self.stack.pop();
                match into {
                    None => panic!("empty stack"),
                    Some(Value::Integer(_)) => panic!("writing into integer"),
                    Some(Value::Memory(address)) => global.write(address, from),
                    Some(Value::Temporary(temporary)) => {
                        self.temporaries.insert(temporary, from);
                    }
                    Some(Value::Label(_)) => panic!("writing into label"),
                }
            }
            Hir::Statement(hir::Statement::Return(r#returns)) => {
                let mut r#returns = (0..r#returns.len())
                    .map(|_| self.pop_integer(global))
                    .collect::<Vec<_>>();

                r#returns.reverse();

                return Some(r#returns);
            }
            Hir::Statement(hir::Statement::Sequence(_)) => unreachable!(),
        }

        self.index += 1;
        None
    }

    fn call(&mut self, unit: &ir::Unit<Flat>, global: &mut Global, call: &hir::Call) -> Vec<i64> {
        let mut arguments = (0..call.arguments.len())
            .map(|_| self.pop_integer(global))
            .collect::<Vec<_>>();

        arguments.reverse();

        let name = match self.pop_label() {
            operand::Label::Fixed(name) => name,
            operand::Label::Fresh(_, _) => panic!("calling fresh function name"),
        };

        self.library(global, name, &arguments)
            .unwrap_or_else(|| global.call(unit, &name, &arguments))
    }

    fn library(
        &mut self,
        global: &mut Global,
        name: Symbol,
        arguments: &[i64],
    ) -> Option<Vec<i64>> {
        let r#returns = match symbol::resolve(name) {
            "_Iprint_pai" => {
                assert_eq!(arguments.len(), 1);
                for byte in global.read_array(arguments[0]) {
                    print!("{}", u8::try_from(*byte).unwrap() as char);
                }
                Vec::new()
            }
            "_Iprintln_pai" => {
                assert_eq!(arguments.len(), 1);
                for byte in global.read_array(arguments[0]) {
                    print!("{}", u8::try_from(*byte).unwrap() as char);
                }
                println!();
                Vec::new()
            }
            "_Ireadln_ai" => {
                assert_eq!(arguments.len(), 0);
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();
                assert_eq!(buffer.pop(), Some('\n'));
                vec![global.write_array(buffer.as_bytes())]
            }
            "_Igetchar_i" => {
                assert_eq!(arguments.len(), 0);
                let mut char = [0];
                std::io::stdin().read_exact(&mut char).unwrap();
                vec![char[0] as i64]
            }
            "_Ieof_b" => unimplemented!(),
            "_IunparseInt_aii" => {
                assert_eq!(arguments.len(), 1);
                vec![global.write_array(arguments[0].to_string().as_bytes())]
            }
            "_IparseInt_t2ibai" => {
                assert_eq!(arguments.len(), 1);

                let integer = global
                    .read_array(arguments[0])
                    .iter()
                    .map(|byte| u8::try_from(*byte).unwrap() as char)
                    .collect::<String>()
                    .parse::<i64>();

                match integer {
                    Ok(integer) => vec![integer, 1],
                    Err(_) => vec![0, 0],
                }
            }
            "_xi_alloc" => {
                assert_eq!(arguments.len(), 1);
                vec![global.calloc(arguments[0])]
            }
            "_xi_out_of_bounds" => panic!("out of bounds"),
            "_Iassert_pb" => {
                assert_eq!(arguments.len(), 1);
                if arguments[0] != 1 {
                    panic!("Assertion error");
                }
                Vec::new()
            }
            _ => return None,
        };

        Some(r#returns)
    }

    fn pop_integer(&mut self, global: &Global) -> i64 {
        match self.stack.pop() {
            None => panic!("empty stack"),
            Some(Value::Integer(integer)) => integer,
            Some(Value::Memory(address)) => global.read(address),
            Some(Value::Label(_)) => panic!("using label as integer"),
            Some(Value::Temporary(temporary)) => self.temporaries[&temporary],
        }
    }

    fn pop_label(&mut self) -> operand::Label {
        match self.stack.pop() {
            None => panic!("empty stack"),
            Some(Value::Integer(_)) => panic!("using integer as label"),
            Some(Value::Memory(_)) => panic!("using memory as label"),
            Some(Value::Label(label)) => label,
            Some(Value::Temporary(_)) => panic!("using temporary as label"),
        }
    }
}

impl Global {
    fn read(&self, address: i64) -> i64 {
        let index = Self::index(address);
        self.heap.get(index).copied().unwrap()
    }

    fn write(&mut self, address: i64, value: i64) {
        let index = Self::index(address);
        self.heap[index] = value;
    }

    fn read_array(&self, address: i64) -> &[i64] {
        let len = self.read(address - constants::WORD_SIZE);
        assert!(len >= 0);
        let index = Self::index(address);
        &self.heap[index..][..len as usize]
    }

    fn write_array(&mut self, array: &[u8]) -> i64 {
        let len = array.len() as i64;
        let address = self.malloc((len + 1) * constants::WORD_SIZE);

        self.write(address, len);

        for (index, byte) in array.iter().copied().enumerate() {
            self.write(
                address + (index as i64 + 1) * constants::WORD_SIZE,
                byte as i64,
            );
        }

        address + constants::WORD_SIZE
    }

    fn index(address: i64) -> usize {
        let address = usize::try_from(address).unwrap();

        if address % constants::WORD_SIZE as usize > 0 {
            panic!("Unaligned memory access: {:x}", address);
        }

        address / constants::WORD_SIZE as usize
    }

    fn malloc(&mut self, bytes: i64) -> i64 {
        if bytes < 0 {
            todo!()
        }

        if bytes % constants::WORD_SIZE > 0 {
            todo!()
        }

        let index = self.heap.len() as i64;

        if index * constants::WORD_SIZE + bytes > HEAP_SIZE as i64 {
            todo!()
        }

        for _ in 0..bytes / constants::WORD_SIZE {
            self.heap.push(self.rng.gen());
        }

        index * constants::WORD_SIZE
    }

    fn calloc(&mut self, bytes: i64) -> i64 {
        let address = self.malloc(bytes);
        let index = Self::index(address);
        for offset in index..(bytes / constants::WORD_SIZE) as usize {
            self.heap[index + offset] = 0;
        }
        address
    }
}

#[derive(Default)]
struct Flat<'a> {
    instructions: Vec<Hir<'a>>,
    labels: BTreeMap<operand::Label, usize>,
}

#[derive(Copy, Clone, Debug)]
enum Hir<'a> {
    Expression(&'a hir::Expression),
    Statement(&'a hir::Statement),
}

impl<'a> Flat<'a> {
    fn flatten_function(function: &'a hir::Function) -> Self {
        let mut flat = Flat::default();
        flat.flatten_statement(&function.statements);
        flat
    }

    fn flatten_expression(&mut self, expression: &'a hir::Expression) {
        match expression {
            hir::Expression::Integer(_)
            | hir::Expression::Label(_)
            | hir::Expression::Temporary(_) => (),
            hir::Expression::Memory(address) => {
                self.flatten_expression(address);
            }
            hir::Expression::Binary(_, left, right) => {
                self.flatten_expression(left);
                self.flatten_expression(right);
            }
            hir::Expression::Call(call) => self.flatten_call(call),
            hir::Expression::Sequence(statement, expression) => {
                self.flatten_statement(statement);
                self.flatten_expression(expression);
                return;
            }
        }

        self.instructions.push(Hir::Expression(expression));
    }

    fn flatten_statement(&mut self, statement: &'a hir::Statement) {
        match statement {
            hir::Statement::Jump(expression) => self.flatten_expression(expression),
            hir::Statement::CJump(condition, _, _) => self.flatten_expression(condition),
            hir::Statement::Label(label) => {
                self.labels.insert(*label, self.instructions.len());
                return;
            }
            hir::Statement::Call(call) => self.flatten_call(call),
            hir::Statement::Move(into, from) => {
                self.flatten_expression(into);
                self.flatten_expression(from);
            }
            hir::Statement::Return(returns) => {
                returns
                    .iter()
                    .for_each(|r#return| self.flatten_expression(r#return));
            }
            hir::Statement::Sequence(statements) => {
                statements
                    .iter()
                    .for_each(|statement| self.flatten_statement(statement));
                return;
            }
        }

        self.instructions.push(Hir::Statement(statement));
    }

    fn flatten_call(&mut self, hir::Call { name, arguments }: &'a hir::Call) {
        self.flatten_expression(name);
        arguments
            .iter()
            .for_each(|argument| self.flatten_expression(argument));
    }
}
