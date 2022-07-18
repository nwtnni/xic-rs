use std::io::BufRead;
use std::io::Read;
use std::io::Write;

use anyhow::anyhow;
use rand::rngs::ThreadRng;
use rand::Rng as _;

use crate::abi;
use crate::data::ir::Linkage;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::interpret::Value;
use crate::Map;

const HEAP_SIZE: usize = 64 * 1024;

pub struct Global<'io> {
    data: Map<Label, Vec<Value>>,
    heap: Vec<Value>,
    rng: ThreadRng,
    stdin: Box<dyn BufRead + 'io>,
    stdout: Box<dyn Write + 'io>,
}

impl<'io> Global<'io> {
    pub fn new<R: BufRead + 'io, W: Write + 'io>(
        data: &Map<Label, Vec<Immediate>>,
        bss: &Map<Symbol, (Linkage, usize)>,
        stdin: R,
        stdout: W,
    ) -> Self {
        let mut r#static = Map::default();

        for (label, data) in data {
            r#static.insert(
                *label,
                data.iter()
                    .map(|immediate| match immediate {
                        Immediate::Integer(integer) => Value::Integer(*integer),
                        Immediate::Label(label) => Value::Label(*label, 0),
                    })
                    .collect(),
            );
        }

        for (symbol, (_, size)) in bss {
            r#static.insert(Label::Fixed(*symbol), vec![Value::Integer(0); *size]);
        }

        Global {
            data: r#static,
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
            abi::XI_PRINT => {
                debug_assert_eq!(arguments.len(), 1);

                // Note: this is a false positive, as `self.read_array(...)` borrows
                // all of `self` immutably, and then we try to write to `&mut self.stdout`.
                //
                // This actually wouldn't be a problem if the code were inlined, since
                // we only borrow `self.heap` or `self.data`, but Rust can't reason about
                // partial borrows across function boundaries right now.
                #[allow(clippy::unnecessary_to_owned)]
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
            abi::XI_PRINTLN => {
                debug_assert_eq!(arguments.len(), 1);
                #[allow(clippy::unnecessary_to_owned)]
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
            abi::XI_READLN => {
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
            abi::XI_GETCHAR => {
                debug_assert_eq!(arguments.len(), 0);
                let mut char = [0];
                self.stdin.read_exact(&mut char).unwrap();
                vec![Value::Integer(char[0] as i64)]
            }
            abi::XI_EOF => unimplemented!(),
            abi::XI_UNPARSE_INT => {
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
            abi::XI_PARSE_INT => {
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
            abi::XI_ALLOC => {
                debug_assert_eq!(arguments.len(), 1);
                vec![self.calloc(arguments[0])]
            }
            abi::XI_OUT_OF_BOUNDS => panic!("out of bounds"),
            abi::XI_ASSERT => {
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
        log::debug!("Reading memory at address {:?}", address);
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
        log::debug!(
            "Writing value {:?} to memory at address {:?}",
            value,
            address
        );
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
        log::debug!("Reading array from memory at address {:?}", address);
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
        log::debug!("Writing array {:?} to memory", array);
        let len = array.len() as i64;
        let address = self
            .malloc(Value::Integer((len + 1) * abi::WORD))
            .into_integer();

        self.write(Value::Integer(address), Value::Integer(len));

        for (index, value) in array.iter().copied().enumerate() {
            self.write(
                Value::Integer(address + (index as i64 + 1) * abi::WORD),
                value,
            );
        }

        Value::Integer(address + abi::WORD)
    }

    fn index(address: i64) -> usize {
        let address = usize::try_from(address).unwrap();

        if address % abi::WORD as usize > 0 {
            panic!("Unaligned memory access: {:x}", address);
        }

        address / abi::WORD as usize
    }

    fn malloc(&mut self, bytes: Value) -> Value {
        log::debug!("Calling malloc for {:?} bytes", bytes);
        let bytes = bytes.into_integer();

        if bytes < 0 {
            todo!()
        }

        if bytes % abi::WORD > 0 {
            todo!()
        }

        let index = self.heap.len() as i64;

        if index * abi::WORD + bytes > HEAP_SIZE as i64 {
            todo!()
        }

        for _ in 0..bytes / abi::WORD {
            self.heap.push(Value::Integer(self.rng.gen()));
        }

        Value::Integer(index * abi::WORD)
    }

    pub fn calloc(&mut self, bytes: Value) -> Value {
        let address = self.malloc(bytes).into_integer();
        let index = Self::index(address);
        for offset in index..(bytes.into_integer() / abi::WORD) as usize {
            self.heap[index + offset] = Value::Integer(0);
        }
        Value::Integer(address)
    }
}
