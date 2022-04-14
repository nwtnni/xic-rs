use std::collections::BTreeMap;
use std::convert::TryFrom as _;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;

use anyhow::anyhow;
use rand::rngs::ThreadRng;
use rand::Rng as _;

use crate::constants;
use crate::data::operand;
use crate::interpret::Value;
use crate::util::symbol;
use crate::util::symbol::Symbol;

const HEAP_SIZE: usize = 1024;

pub struct Global {
    data: BTreeMap<operand::Label, Vec<Value>>,
    heap: Vec<Value>,
    rng: ThreadRng,
    stdin: Box<dyn BufRead>,
    stdout: Box<dyn Write>,
}

impl Global {
    pub fn new<R: BufRead + 'static, W: Write + 'static>(
        data: &BTreeMap<Symbol, operand::Label>,
        stdin: R,
        stdout: W,
    ) -> Self {
        Global {
            data: data
                .iter()
                .map(|(symbol, label)| {
                    let mut string = symbol::resolve(*symbol)
                        .bytes()
                        .map(|byte| byte as i64)
                        .map(Value::Integer)
                        .collect::<Vec<_>>();
                    string.insert(0, Value::Integer(string.len() as i64));
                    (*label, string)
                })
                .collect(),
            heap: Vec::new(),
            rng: rand::thread_rng(),
            stdin: Box::new(stdin),
            stdout: Box::new(stdout),
        }
    }

    pub fn interpret_library(
        &mut self,
        name: Symbol,
        arguments: &[Value],
    ) -> Option<anyhow::Result<Vec<Value>>> {
        let r#returns = match symbol::resolve(name) {
            "_Iprint_pai" => {
                debug_assert_eq!(arguments.len(), 1);
                for byte in self.read_array(arguments[0]).to_vec() {
                    write!(
                        &mut self.stdout,
                        "{}",
                        u8::try_from(byte.into_integer()).unwrap() as char
                    )
                    .unwrap();
                }
                self.stdout.flush().unwrap();
                Vec::new()
            }
            "_Iprintln_pai" => {
                debug_assert_eq!(arguments.len(), 1);
                for byte in self.read_array(arguments[0]).to_vec() {
                    write!(
                        &mut self.stdout,
                        "{}",
                        u8::try_from(byte.into_integer()).unwrap() as char
                    )
                    .unwrap();
                }
                writeln!(&mut self.stdout).unwrap();
                self.stdout.flush().unwrap();
                Vec::new()
            }
            "_Ireadln_ai" => {
                debug_assert_eq!(arguments.len(), 0);
                let mut buffer = String::new();
                self.stdin.read_line(&mut buffer).unwrap();
                debug_assert_eq!(buffer.pop(), Some('\n'));
                vec![self.write_array(
                    &buffer
                        .bytes()
                        .map(|byte| byte as i64)
                        .map(Value::Integer)
                        .collect::<Vec<_>>(),
                )]
            }
            "_Igetchar_i" => {
                debug_assert_eq!(arguments.len(), 0);
                let mut char = [0];
                self.stdin.read_exact(&mut char).unwrap();
                vec![Value::Integer(char[0] as i64)]
            }
            "_Ieof_b" => unimplemented!(),
            "_IunparseInt_aii" => {
                debug_assert_eq!(arguments.len(), 1);
                vec![self.write_array(
                    &arguments[0]
                        .into_integer()
                        .to_string()
                        .bytes()
                        .map(|byte| byte as i64)
                        .map(Value::Integer)
                        .collect::<Vec<_>>(),
                )]
            }
            "_IparseInt_t2ibai" => {
                debug_assert_eq!(arguments.len(), 1);

                let integer = self
                    .read_array(arguments[0])
                    .iter()
                    .map(|byte| u8::try_from(byte.into_integer()).unwrap() as char)
                    .collect::<String>()
                    .parse::<i64>();

                match integer {
                    Ok(integer) => vec![Value::Integer(integer), Value::Integer(1)],
                    Err(_) => vec![Value::Integer(0), Value::Integer(0)],
                }
            }
            "_xi_alloc" => {
                debug_assert_eq!(arguments.len(), 1);
                vec![self.calloc(arguments[0])]
            }
            "_xi_out_of_bounds" => panic!("out of bounds"),
            "_Iassert_pb" => {
                debug_assert_eq!(arguments.len(), 1);
                if arguments[0].into_integer() != 1 {
                    return Some(Err(anyhow!("Assertion error: {:?}", arguments[0])));
                }
                Vec::new()
            }
            _ => return None,
        };

        Some(Ok(r#returns))
    }

    pub fn read(&self, address: Value) -> Value {
        match address {
            Value::Integer(address) => {
                let index = Self::index(address);
                self.heap.get(index).copied().unwrap()
            }
            Value::Label(label, offset) => {
                let index = Self::index(offset);
                self.data.get(&label).unwrap().get(index).copied().unwrap()
            }
        }
    }

    pub fn write(&mut self, address: Value, value: Value) {
        match address {
            Value::Integer(address) => {
                let index = Self::index(address);
                *self.heap.get_mut(index).unwrap() = value;
            }
            Value::Label(label, offset) => {
                let index = Self::index(offset);
                *self.data.get_mut(&label).unwrap().get_mut(index).unwrap() = value;
            }
        }
    }

    pub fn read_array(&self, address: Value) -> &[Value] {
        match address {
            Value::Integer(address) => {
                let index = Self::index(address);
                let len = match self.heap[index - 1] {
                    Value::Integer(len) => len,
                    Value::Label(_, _) => panic!("stored len as label"),
                };
                &self.heap[index..][..len as usize]
            }
            Value::Label(label, 8) => {
                let len = match self.data.get(&label).unwrap().first().unwrap() {
                    Value::Integer(len) => len,
                    Value::Label(_, _) => panic!("stored len as label"),
                };
                &self.data.get(&label).unwrap()[1..][..*len as usize]
            }
            Value::Label(_, _) => panic!("reading array at non-zero label offset"),
        }
    }

    pub fn write_array(&mut self, array: &[Value]) -> Value {
        let len = array.len() as i64;
        let address = self
            .malloc(Value::Integer((len + 1) * constants::WORD_SIZE))
            .into_integer();

        self.write(Value::Integer(address), Value::Integer(len));

        for (index, value) in array.iter().copied().enumerate() {
            self.write(
                Value::Integer(address + (index as i64 + 1) * constants::WORD_SIZE),
                value,
            );
        }

        Value::Integer(address + constants::WORD_SIZE)
    }

    fn index(address: i64) -> usize {
        let address = usize::try_from(address).unwrap();

        if address % constants::WORD_SIZE as usize > 0 {
            panic!("Unaligned memory access: {:x}", address);
        }

        address / constants::WORD_SIZE as usize
    }

    fn malloc(&mut self, bytes: Value) -> Value {
        let bytes = bytes.into_integer();

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
            self.heap.push(Value::Integer(self.rng.gen()));
        }

        Value::Integer(index * constants::WORD_SIZE)
    }

    pub fn calloc(&mut self, bytes: Value) -> Value {
        let address = self.malloc(bytes).into_integer();
        let index = Self::index(address);
        for offset in index..(bytes.into_integer() / constants::WORD_SIZE) as usize {
            self.heap[index + offset] = Value::Integer(0);
        }
        Value::Integer(address)
    }
}