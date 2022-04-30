#![allow(dead_code)]

use std::collections::BTreeMap;
use std::iter;
use std::slice;

use crate::abi;
use crate::data::asm;
use crate::data::asm::Assembly;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::util::Or;

struct Trivial {
    callee_arguments: usize,
    callee_returns: usize,
    temporaries: BTreeMap<Temporary, usize>,
    instructions: Vec<Assembly<Register>>,
    unused: iter::Rev<slice::Iter<'static, Register>>,
}

fn allocate_function(function: &asm::Function<Temporary>) -> Vec<Assembly<Register>> {
    let mut trivial = Trivial {
        callee_arguments: function.callee_arguments,
        callee_returns: function.callee_returns,
        temporaries: BTreeMap::new(),
        instructions: Vec::new(),
        unused: abi::CALLER_SAVED.iter().rev(),
    };

    for instruction in &function.instructions {
        trivial.allocate_instruction(instruction);
    }

    let stack_size = abi::stack_size(
        function.callee_arguments,
        function.callee_returns,
        trivial.temporaries.len(),
    ) as i64;

    // Prologue
    trivial.instructions.insert(
        0,
        Assembly::Binary(
            asm::Binary::Sub,
            operand::Binary::RI {
                destination: Register::Rsp,
                source: Immediate::Integer(stack_size),
            },
        ),
    );

    // Epilogue
    trivial.instructions.extend([
        Assembly::Binary(
            asm::Binary::Add,
            operand::Binary::RI {
                destination: Register::Rsp,
                source: Immediate::Integer(stack_size),
            },
        ),
        Assembly::Nullary(asm::Nullary::Ret),
    ]);

    trivial.instructions
}

impl Trivial {
    fn allocate_instruction(&mut self, instruction: &Assembly<Temporary>) {
        self.unused = abi::CALLER_SAVED.iter().rev();

        let instruction = match instruction {
            Assembly::Binary(binary, operands) => {
                Assembly::Binary(*binary, self.allocate_binary(operands))
            }
            Assembly::Unary(unary, operand) => {
                Assembly::Unary(*unary, self.allocate_unary(operand))
            }
            Assembly::Nullary(nullary) => Assembly::Nullary(*nullary),
            Assembly::Label(label) => Assembly::Label(*label),
        };

        self.instructions.push(instruction);
    }

    fn allocate_binary(
        &mut self,
        binary: &operand::Binary<Temporary>,
    ) -> operand::Binary<Register> {
        let (destination, source) = match binary {
            operand::Binary::RI {
                destination,
                source,
            } => {
                return match self.allocate(destination) {
                    Or::L(register) => operand::Binary::RI {
                        destination: register,
                        source: *source,
                    },
                    Or::R(memory) => operand::Binary::MI {
                        destination: memory,
                        source: *source,
                    },
                }
            }
            operand::Binary::MI {
                destination,
                source,
            } => {
                return operand::Binary::MI {
                    destination: self.allocate_memory(destination),
                    source: *source,
                }
            }
            operand::Binary::MR {
                destination,
                source,
            } => (
                Or::R(self.allocate_memory(destination)),
                self.allocate(source),
            ),
            operand::Binary::RM {
                destination,
                source,
            } => (
                self.allocate(destination),
                Or::R(self.allocate_memory(source)),
            ),
            operand::Binary::RR {
                destination,
                source,
            } => (self.allocate(destination), self.allocate(source)),
        };

        match (destination, source) {
            (Or::L(destination), Or::L(source)) => operand::Binary::RR {
                destination,
                source,
            },
            (Or::L(destination), Or::R(source)) => operand::Binary::RM {
                destination,
                source,
            },
            (Or::R(destination), Or::L(source)) => operand::Binary::MR {
                destination,
                source,
            },
            (Or::R(destination), Or::R(source)) => {
                let register = self.unused.next().copied().unwrap();

                self.instructions.push(Assembly::Binary(
                    asm::Binary::Mov,
                    operand::Binary::RM {
                        destination: register,
                        source,
                    },
                ));

                operand::Binary::MR {
                    destination,
                    source: register,
                }
            }
        }
    }

    fn allocate_unary(&mut self, unary: &operand::Unary<Temporary>) -> operand::Unary<Register> {
        match unary {
            operand::Unary::I(immediate) => operand::Unary::I(*immediate),
            operand::Unary::R(temporary) => match self.allocate(temporary) {
                Or::L(register) => operand::Unary::R(register),
                Or::R(memory) => operand::Unary::M(memory),
            },
            operand::Unary::M(memory) => operand::Unary::M(self.allocate_memory(memory)),
        }
    }

    fn allocate_memory(
        &mut self,
        memory: &operand::Memory<Temporary>,
    ) -> operand::Memory<Register> {
        memory.map(|temporary| {
            let memory = match self.allocate(temporary) {
                Or::L(register) => return register,
                Or::R(memory) => memory,
            };

            let register = self.unused.next().copied().unwrap();

            self.instructions.push(Assembly::Binary(
                asm::Binary::Mov,
                operand::Binary::RM {
                    destination: register,
                    source: memory,
                },
            ));

            register
        })
    }

    fn allocate(&mut self, temporary: &Temporary) -> Or<Register, Memory<Register>> {
        if let Temporary::Register(register) = temporary {
            return Or::L(*register);
        }

        let len = self.temporaries.len();
        let index = self.temporaries.entry(*temporary).or_insert(len);

        Or::R(Memory::BO {
            base: Register::Rsp,
            offset: Immediate::Integer(abi::stack_offset(
                self.callee_arguments,
                self.callee_returns,
                *index,
            ) as i64),
        })
    }
}
