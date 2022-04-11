use std::convert::TryFrom as _;
use std::io::Read as _;

use anyhow::anyhow;
use rand::rngs::ThreadRng;
use rand::Rng as _;

use crate::constants;
use crate::data::operand;
use crate::util::symbol;
use crate::util::symbol::Symbol;

const HEAP_SIZE: usize = 1024;

pub struct Global {
    heap: Vec<i64>,
    rng: ThreadRng,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            heap: Vec::new(),
            rng: rand::thread_rng(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Integer(i64),
    Label(operand::Label),
    Memory(i64),
    Temporary(operand::Temporary),
}

impl Global {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret_library(
        &mut self,
        name: Symbol,
        arguments: &[i64],
    ) -> Option<anyhow::Result<Vec<i64>>> {
        let r#returns = match symbol::resolve(name) {
            "_Iprint_pai" => {
                debug_assert_eq!(arguments.len(), 1);
                for byte in self.read_array(arguments[0]) {
                    print!("{}", u8::try_from(*byte).unwrap() as char);
                }
                Vec::new()
            }
            "_Iprintln_pai" => {
                debug_assert_eq!(arguments.len(), 1);
                for byte in self.read_array(arguments[0]) {
                    print!("{}", u8::try_from(*byte).unwrap() as char);
                }
                println!();
                Vec::new()
            }
            "_Ireadln_ai" => {
                debug_assert_eq!(arguments.len(), 0);
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();
                debug_assert_eq!(buffer.pop(), Some('\n'));
                vec![self.write_array(buffer.as_bytes())]
            }
            "_Igetchar_i" => {
                debug_assert_eq!(arguments.len(), 0);
                let mut char = [0];
                std::io::stdin().read_exact(&mut char).unwrap();
                vec![char[0] as i64]
            }
            "_Ieof_b" => unimplemented!(),
            "_IunparseInt_aii" => {
                debug_assert_eq!(arguments.len(), 1);
                vec![self.write_array(arguments[0].to_string().as_bytes())]
            }
            "_IparseInt_t2ibai" => {
                debug_assert_eq!(arguments.len(), 1);

                let integer = self
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
                debug_assert_eq!(arguments.len(), 1);
                vec![self.calloc(arguments[0])]
            }
            "_xi_out_of_bounds" => panic!("out of bounds"),
            "_Iassert_pb" => {
                debug_assert_eq!(arguments.len(), 1);
                if arguments[0] != 1 {
                    return Some(Err(anyhow!("Assertion error: {}", arguments[0])));
                }
                Vec::new()
            }
            _ => return None,
        };

        Some(Ok(r#returns))
    }

    pub fn read(&self, address: i64) -> i64 {
        let index = Self::index(address);
        self.heap.get(index).copied().unwrap()
    }

    pub fn write(&mut self, address: i64, value: i64) {
        let index = Self::index(address);
        self.heap[index] = value;
    }

    pub fn read_array(&self, address: i64) -> &[i64] {
        let len = self.read(address - constants::WORD_SIZE);
        debug_assert!(len >= 0);
        let index = Self::index(address);
        &self.heap[index..][..len as usize]
    }

    pub fn write_array(&mut self, array: &[u8]) -> i64 {
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

    pub fn calloc(&mut self, bytes: i64) -> i64 {
        let address = self.malloc(bytes);
        let index = Self::index(address);
        for offset in index..(bytes / constants::WORD_SIZE) as usize {
            self.heap[index + offset] = 0;
        }
        address
    }
}
