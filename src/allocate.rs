mod linear;
mod trivial;

use std::array;

use crate::abi;
use crate::asm;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Scale;
use crate::data::operand::Temporary;
use crate::util::Or;
use crate::Map;

pub fn allocate_trivial(function: &asm::Function<Temporary>) -> asm::Function<Register> {
    let allocated = Map::default();
    let spilled = trivial::allocate(function);
    allocate(function, allocated, spilled)
}

pub fn allocate_linear(function: Cfg<asm::Function<Temporary>>) -> asm::Function<Register> {
    let (function, allocated, spilled) = linear::allocate(function);
    allocate(&function, allocated, spilled)
}

struct Allocator {
    callee_arguments: usize,
    callee_returns: usize,
    allocated: Map<Temporary, Register>,
    spilled: Map<Temporary, usize>,
    statements: Vec<asm::Statement<Register>>,
    shuttle: array::IntoIter<Register, 2>,
}

// Registers reserved for shuttling spilled temmporaries.
//
// In the worst case, we need at most two registers. For example,
// consider the following abstract assembly statement:
//
// ```text
// mov [t0 + t1 * 8 + label], t2
// ```
//
// Let's say we spill all of the temporaries to the stack.
// Then we end up with this:
//
// ```text
// mov [[rsp + 0] + [rsp + 8] * scale + offset], [rsp + 16]
// ```
//
// It turns out we can actually use a single register to compute
// `[rsp + 0] + [rsp + 8] * scale + offset`, but we need a second
// register to shuttle `[rsp + 16]`, since there's no memory-to-memory
// statement encoding.
const SHUTTLE: [Register; 2] = [Register::R10, Register::R11];

fn allocate(
    function: &asm::Function<Temporary>,
    allocated: Map<Temporary, Register>,
    spilled: Map<Temporary, usize>,
) -> asm::Function<Register> {
    let mut allocator = Allocator {
        callee_arguments: function.callee_arguments().unwrap_or(0),
        callee_returns: function.callee_returns().unwrap_or(0),
        allocated,
        spilled,
        statements: Vec::new(),
        shuttle: SHUTTLE.into_iter(),
    };

    for statement in &function.statements {
        allocator.allocate_statement(statement);
    }

    let stack_size = abi::stack_size(
        allocator.callee_arguments,
        allocator.callee_returns,
        allocator.spilled.len(),
    ) as i64;

    allocator
        .statements
        .iter_mut()
        .for_each(|statement| rewrite_rbp(stack_size, statement));

    assert!(matches!(
        allocator.statements.first(),
        Some(asm::Statement::Label(label)) if *label == function.enter,
    ));

    assert!(matches!(
        allocator.statements.last(),
        Some(asm::Statement::Nullary(asm::Nullary::Ret(returns))) if *returns == function.returns,
    ));

    allocator.statements.insert(1, asm!((sub rsp, stack_size)));

    let len = allocator.statements.len();

    allocator
        .statements
        .insert(len - 1, asm!((add rsp, stack_size)));

    asm::Function {
        name: function.name,
        arguments: function.arguments,
        returns: function.returns,
        statements: allocator.statements,
        global: function.global,
        enter: function.enter,
        exit: function.exit,
    }
}

impl Allocator {
    fn allocate_statement(&mut self, statement: &asm::Statement<Temporary>) {
        self.shuttle = SHUTTLE.into_iter();

        let statement = match statement {
            // Since the linear scan allocator is based on live variable analysis,
            // it doesn't allocate registers for dead variables. This is only allowed
            // for `mov` and `lea` statements, since they don't read their destinations.
            asm::Statement::Binary(
                asm::Binary::Mov | asm::Binary::Lea,
                operand::Binary::RI { destination, .. }
                | operand::Binary::RM { destination, .. }
                | operand::Binary::RR { destination, .. },
            ) if !matches!(destination, Temporary::Register(_))
                && !self.allocated.contains_key(destination)
                && !self.spilled.contains_key(destination) =>
            {
                return;
            }

            // Special case: this is the only statement that can take a 64-bit immediate operand.
            // Tiling guarantees that all 64-bit uses will be shuttled into a move like this one.
            &asm::Statement::Binary(
                asm::Binary::Mov,
                operand::Binary::RI {
                    destination,
                    source,
                },
            ) if source.is_64_bit() => match self.allocate(&destination) {
                Or::L(register) => asm!((mov register, source)),
                Or::R(memory) => {
                    let shuttle = self.shuttle.next().unwrap();
                    self.statements.push(asm!((mov shuttle, source)));
                    asm!((mov memory, shuttle))
                }
            },

            // Special case: `imul` can only take a register destination, so we need
            // to shuttle any memory destinations.
            asm::Statement::Binary(asm::Binary::Mul, operands) => {
                match self.allocate_binary(operands) {
                    operands @ (operand::Binary::RI { .. }
                    | operand::Binary::RM { .. }
                    | operand::Binary::RR { .. }) => {
                        asm::Statement::Binary(asm::Binary::Mul, operands)
                    }
                    operand::Binary::MI {
                        destination,
                        source,
                    } => {
                        let shuttle = self.shuttle.next().unwrap();
                        self.statements.push(asm!((mov shuttle, destination)));
                        self.statements.push(asm!((imul shuttle, source)));
                        asm!((mov destination, shuttle))
                    }
                    operand::Binary::MR {
                        destination,
                        source,
                    } => {
                        // FIXME: this actually requires three shuttle registers at worst.
                        //
                        // We can work around this with the `xchg` instruction, relying on
                        // the commutativity of multiplication, but we need to implement
                        // it first.
                        let shuttle = self.shuttle.next().unwrap();
                        self.statements.push(asm!((mov shuttle, destination)));
                        self.statements.push(asm!((imul shuttle, source)));
                        asm!((mov destination, shuttle))
                    }
                }
            }

            asm::Statement::Binary(binary, operands) => {
                asm::Statement::Binary(*binary, self.allocate_binary(operands))
            }
            asm::Statement::Unary(unary, operand) => {
                asm::Statement::Unary(*unary, self.allocate_unary(operand))
            }
            asm::Statement::Nullary(nullary) => asm::Statement::Nullary(*nullary),
            asm::Statement::Label(label) => asm::Statement::Label(*label),
            asm::Statement::Jmp(label) => asm::Statement::Jmp(*label),
            asm::Statement::Jcc(condition, label) => asm::Statement::Jcc(*condition, *label),
        };

        match statement {
            // Omit now-redundant moves
            asm::Statement::Binary(
                asm::Binary::Mov,
                operand::Binary::RR {
                    destination,
                    source,
                },
            ) if destination == source => (),
            // Omit nops
            asm::Statement::Nullary(asm::Nullary::Nop) => (),
            statement => self.statements.push(statement),
        }
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
                    Or::L(register) => operand::Binary::from((register, *source)),
                    Or::R(memory) => operand::Binary::from((memory, *source)),
                }
            }
            operand::Binary::MI {
                destination,
                source,
            } => return operand::Binary::from((self.allocate_memory(destination), *source)),
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
            (Or::L(destination), Or::L(source)) => operand::Binary::from((destination, source)),
            (Or::L(register), Or::R(memory)) => operand::Binary::from((register, memory)),
            (Or::R(memory), Or::L(register)) => operand::Binary::from((memory, register)),
            (Or::R(destination), Or::R(source)) => {
                let source = self.shuttle(asm::Binary::Mov, None, source);
                operand::Binary::from((destination, source))
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
        let (base, index, scale, offset) = match memory {
            Memory::O { offset } => return Memory::O { offset: *offset },
            Memory::B { base } => {
                return Memory::B {
                    base: match self.allocate(base) {
                        Or::L(register) => register,
                        Or::R(memory) => self.shuttle(asm::Binary::Mov, None, memory),
                    },
                }
            }
            Memory::BO { base, offset } => {
                return Memory::BO {
                    base: match self.allocate(base) {
                        Or::L(register) => register,
                        Or::R(memory) => self.shuttle(asm::Binary::Mov, None, memory),
                    },
                    offset: *offset,
                };
            }
            Memory::ISO {
                index,
                scale,
                offset,
            } => {
                return Memory::ISO {
                    index: match self.allocate(index) {
                        Or::L(register) => register,
                        Or::R(memory) => self.shuttle(asm::Binary::Mov, None, memory),
                    },
                    scale: *scale,
                    offset: *offset,
                }
            }
            Memory::BI { base, index } => (base, index, None, None),
            Memory::BIO {
                base,
                index,
                offset,
            } => (base, index, None, Some(*offset)),
            Memory::BIS { base, index, scale } => (base, index, Some(*scale), None),
            Memory::BISO {
                base,
                index,
                scale,
                offset,
            } => (base, index, Some(*scale), Some(*offset)),
        };

        let (base, index) = match (self.allocate(base), self.allocate(index), scale, offset) {
            (Or::L(base), Or::L(index), None, None) => return Memory::BI { base, index },
            (Or::L(base), Or::L(index), Some(scale), None) => {
                return Memory::BIS { base, index, scale }
            }
            (Or::L(base), Or::L(index), None, Some(offset)) => {
                return Memory::BIO {
                    base,
                    index,
                    offset,
                }
            }
            (Or::L(base), Or::L(index), Some(scale), Some(offset)) => {
                return Memory::BISO {
                    base,
                    index,
                    scale,
                    offset,
                }
            }
            (base, index, _, _) => (base, index),
        };

        // Write destructively into the same shuttle register. Index first because
        // we might need to multiply it, and because addition is commutative.
        let shuttle = self.shuttle(asm::Binary::Mov, None, index);

        let shift = match scale {
            None | Some(Scale::_1) => None,
            Some(Scale::_2) => Some(1),
            Some(Scale::_4) => Some(2),
            Some(Scale::_8) => Some(3),
        };

        if let Some(shift) = shift {
            self.shuttle(asm::Binary::Shl, Some(shuttle), shift);
        }

        if let Some(offset) = offset {
            self.shuttle(asm::Binary::Add, Some(shuttle), offset);
        }

        operand::Memory::B {
            base: self.shuttle(asm::Binary::Add, Some(shuttle), base),
        }
    }

    fn shuttle<S: Into<operand::Unary<Register>>>(
        &mut self,
        binary: asm::Binary,
        destination: Option<Register>,
        source: S,
    ) -> Register {
        let destination = destination.unwrap_or_else(|| self.shuttle.next().unwrap());
        let operands = match source.into() {
            operand::Unary::R(register) => operand::Binary::from((destination, register)),
            operand::Unary::M(memory) => operand::Binary::from((destination, memory)),
            operand::Unary::I(immediate) => operand::Binary::from((destination, immediate)),
        };
        self.statements
            .push(asm::Statement::Binary(binary, operands));
        destination
    }

    fn allocate(&self, temporary: &Temporary) -> Or<Register, Memory<Register>> {
        if let Temporary::Register(register) = temporary {
            return Or::L(*register);
        }

        if let Some(register) = self.allocated.get(temporary) {
            return Or::L(*register);
        }

        Or::R(Memory::BO {
            base: Register::rsp(),
            offset: Immediate::Integer(abi::stack_offset(
                self.callee_arguments,
                self.callee_returns,
                self.spilled.get(temporary).copied().unwrap(),
            ) as i64),
        })
    }
}

// We should only tile `[rbp + offset]` when returning multiple arguments.
//
// This needs to be rewritten in terms of `rsp` after the stack size is
// computed, since we don't keep around `rbp` within the function.
fn rewrite_rbp(stack_size: i64, statement: &mut asm::Statement<Register>) {
    let memory = match statement {
        asm::Statement::Binary(_, operand::Binary::RI { .. })
        | asm::Statement::Binary(_, operand::Binary::RR { .. })
        | asm::Statement::Unary(_, operand::Unary::R(_))
        | asm::Statement::Unary(_, operand::Unary::I(_))
        | asm::Statement::Nullary(_)
        | asm::Statement::Label(_)
        | asm::Statement::Jmp(_)
        | asm::Statement::Jcc(_, _) => return,
        #[rustfmt::skip]
        asm::Statement::Binary(_, operand::Binary::MI { destination: memory, .. })
        | asm::Statement::Binary( _, operand::Binary::MR { destination: memory, .. })
        | asm::Statement::Binary(_, operand::Binary::RM { source: memory, .. })
        | asm::Statement::Unary(_, operand::Unary::M(memory)) => memory,
    };

    if let Memory::BO {
        base: base @ Register::Rsp(false),
        offset,
    }
    | Memory::BIO {
        base: base @ Register::Rsp(false),
        index: _,
        offset,
    }
    | Memory::BISO {
        base: base @ Register::Rsp(false),
        index: _,
        scale: _,
        offset,
    } = memory
    {
        *base = Register::Rsp(true);
        match offset {
            Immediate::Label(_) => unreachable!(),
            Immediate::Integer(offset) => *offset += stack_size,
        }
    }
}
