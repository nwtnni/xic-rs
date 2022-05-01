use std::collections::BTreeMap;
use std::convert::TryFrom as _;
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

pub fn allocate_unit(unit: &asm::Unit<Temporary>) -> asm::Unit<Register> {
    unit.map(allocate_function)
}

fn allocate_function(function: &asm::Function<Temporary>) -> asm::Function<Register> {
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

    trivial
        .instructions
        .iter_mut()
        .for_each(|instruction| rewrite_rbp(stack_size, instruction));

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

    asm::Function {
        name: function.name,
        arguments: function.arguments,
        returns: function.returns,
        callee_arguments: function.callee_arguments,
        callee_returns: function.callee_returns,
        instructions: trivial.instructions,
    }
}

// We should only tile `[rbp + offset]` when returning multiple arguments.
//
// This needs to be rewritten in terms of `rsp` after the stack size is
// computed, since we don't keep around `rbp` within the function.
fn rewrite_rbp(stack_size: i64, instruction: &mut Assembly<Register>) {
    let memory = match instruction {
        Assembly::Binary(_, operand::Binary::RI { .. })
        | Assembly::Binary(_, operand::Binary::RR { .. })
        | Assembly::Unary(_, operand::Unary::R(_))
        | Assembly::Unary(_, operand::Unary::I(_))
        | Assembly::Nullary(_)
        | Assembly::Label(_) => return,
        #[rustfmt::skip]
        Assembly::Binary(_, operand::Binary::MI { destination: memory, .. })
        | Assembly::Binary( _, operand::Binary::MR { destination: memory, .. })
        | Assembly::Binary(_, operand::Binary::RM { source: memory, .. })
        | Assembly::Unary(_, operand::Unary::M(memory)) => memory,
    };

    if let Memory::BO {
        base: base @ Register::RspPlaceholder,
        offset,
    }
    | Memory::BIO {
        base: base @ Register::RspPlaceholder,
        index: _,
        offset,
    }
    | Memory::BISO {
        base: base @ Register::RspPlaceholder,
        index: _,
        scale: _,
        offset,
    } = memory
    {
        *base = Register::Rsp;
        match offset {
            Immediate::Label(_) => unreachable!(),
            Immediate::Integer(offset) => *offset += stack_size,
        }
    }
}

impl Trivial {
    fn allocate_instruction(&mut self, instruction: &Assembly<Temporary>) {
        self.unused = abi::CALLER_SAVED.iter().rev();

        let instruction = match instruction {
            // This is the only instruction that can't take a memory destination, so
            // we can't write directly to a stack position.
            Assembly::Binary(
                asm::Binary::Mov,
                operand::Binary::RI {
                    destination,
                    source: Immediate::Integer(integer),
                },
            ) if i32::try_from(*integer).is_err() => {
                let register = self.unused.next().copied().unwrap();

                self.instructions.push(Assembly::Binary(
                    asm::Binary::Mov,
                    operand::Binary::RI {
                        destination: register,
                        source: Immediate::Integer(*integer),
                    },
                ));

                Assembly::Binary(
                    asm::Binary::Mov,
                    self.allocate_binary(&operand::Binary::RR {
                        destination: *destination,
                        source: Temporary::Register(register),
                    }),
                )
            }
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
