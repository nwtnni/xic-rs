mod check;
mod linear;
mod trivial;

use std::iter;
use std::slice;

use crate::abi;
use crate::analyze::analyze;
use crate::analyze::LiveRanges;
use crate::analyze::LiveVariables;
use crate::asm;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Scale;
use crate::data::operand::Temporary;
use crate::optimize;
use crate::util;
use crate::util::Or;
use crate::Map;

pub fn allocate_trivial(function: &asm::Function<Temporary>) -> asm::Function<Register> {
    log::info!(
        "[{}] Allocating {} using trivial algorithm...",
        std::any::type_name::<asm::Function<Temporary>>(),
        function.name,
    );
    util::time!(
        "[{}] Done allocating {}",
        std::any::type_name::<asm::Function<Temporary>>(),
        function.name,
    );

    let allocated = Map::default();
    let spilled = trivial::allocate(function);
    let frame_pointer = match function.statements.get(1) {
        Some(asm::Statement::Unary(
            asm::Unary::Push,
            operand::Unary::R(Temporary::Register(Register::Rbp)),
        )) => abi::FramePointer::Keep,
        _ => abi::FramePointer::Omit,
    };

    allocate(
        frame_pointer,
        &[Register::R11, Register::R10, Register::R9, Register::R8],
        allocated,
        spilled,
        function,
    )
    .expect("[INTERNAL ERROR]: trivial register allocator ran out of shuttle registers")
}

pub fn allocate_linear(mut function: Cfg<asm::Function<Temporary>>) -> asm::Function<Register> {
    log::info!(
        "[{}] Allocating {} using linear algorithm...",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        function.name(),
    );
    util::time!(
        "[{}] Done allocating {}",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        function.name(),
    );

    let frame_pointer = match function[function.enter()].first() {
        Some(asm::Statement::Unary(
            asm::Unary::Push,
            operand::Unary::R(Temporary::Register(Register::Rbp)),
        )) => abi::FramePointer::Keep,
        _ => abi::FramePointer::Omit,
    };

    let mut registers = abi::CALLEE_SAVED
        .iter()
        .chain(abi::CALLER_SAVED)
        .copied()
        // If omitting frame pointer, then we can use it during register allocation
        .filter(|register| *register != Register::Rbp || frame_pointer == abi::FramePointer::Omit)
        .collect::<Vec<_>>();

    let mut shuttles = Vec::new();

    let live_variables = analyze::<LiveVariables<_>, _>(&function);
    optimize::eliminate_dead_code_assembly(&live_variables, &mut function);
    let live_ranges = LiveRanges::new(&live_variables, function);

    // Try to allocate `function` with successively more shuttle registers until we succeed
    loop {
        let (allocated, spilled) = linear::allocate(&live_ranges, registers.clone());
        match allocate(
            frame_pointer,
            &shuttles,
            allocated,
            spilled,
            &live_ranges.function,
        ) {
            Some(function) => {
                log::debug!(
                    "Allocated {} using {} spill registers",
                    function.name,
                    shuttles.len()
                );
                return function;
            }
            None => {
                let register = registers.pop().expect(
                    "[INTERNAL ERROR]: linear register allocator ran out of shuttle registers",
                );
                shuttles.push(register);
            }
        }
    }
}

struct Allocator<'a> {
    callee_arguments: Option<usize>,
    callee_returns: Option<usize>,
    allocated: Map<Temporary, Register>,
    spilled: Map<Temporary, usize>,
    statements: Vec<asm::Statement<Register>>,

    // All registers reserved for shuttling
    shuttle_reserved: &'a [Register],

    // Shuttle registers yet unused in current statement
    shuttle_unused: iter::Copied<slice::Iter<'a, Register>>,
}

fn allocate(
    frame_pointer: abi::FramePointer,
    shuttle: &[Register],
    allocated: Map<Temporary, Register>,
    spilled: Map<Temporary, usize>,
    function: &asm::Function<Temporary>,
) -> Option<asm::Function<Register>> {
    let mut allocator = Allocator {
        callee_arguments: function.callee_arguments(),
        callee_returns: function.callee_returns(),
        allocated,
        spilled,
        statements: Vec::new(),
        shuttle_reserved: shuttle,
        shuttle_unused: shuttle.iter().copied(),
    };

    for statement in &function.statements {
        allocator.allocate_statement(statement)?;
    }

    let stack_size = abi::stack_size(
        frame_pointer,
        allocator.callee_arguments,
        allocator.callee_returns,
        allocator.spilled.len(),
    ) as i64;

    allocator
        .statements
        .iter_mut()
        .for_each(|statement| rewrite_rbp(frame_pointer, stack_size, statement));

    assert!(matches!(
        allocator.statements.first(),
        Some(asm::Statement::Label(label)) if *label == function.enter,
    ));

    assert!(matches!(
        allocator.statements.last(),
        Some(asm::Statement::Nullary(asm::Nullary::Ret(returns))) if *returns == function.returns,
    ));

    if stack_size > 0 {
        allocator.statements.insert(
            match frame_pointer {
                // Place after prologue:
                //
                // ```text
                // push rbp
                // mov rbp, rsp
                // ```
                abi::FramePointer::Keep => 3,
                abi::FramePointer::Omit => 1,
            },
            asm!((sub rsp, stack_size)),
        );

        let len = allocator.statements.len();

        allocator.statements.insert(
            match frame_pointer {
                // Place before epilogue:
                //
                // ```text
                // pop rbp
                // ```
                abi::FramePointer::Keep => len - 2,
                abi::FramePointer::Omit => len - 1,
            },
            asm!((add rsp, stack_size)),
        );
    }

    Some(asm::Function {
        name: function.name,
        arguments: function.arguments,
        returns: function.returns,
        statements: allocator.statements,
        linkage: function.linkage,
        enter: function.enter,
        exit: function.exit,
    })
}

impl<'a> Allocator<'a> {
    fn allocate_statement(&mut self, statement: &asm::Statement<Temporary>) -> Option<()> {
        self.shuttle_unused = self.shuttle_reserved.iter().copied();

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
                return Some(());
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
                    let shuttle = self.shuttle_unused.next()?;
                    self.statements.push(asm!((mov shuttle, source)));
                    asm!((mov memory, shuttle))
                }
            },

            // Special case: `imul` can only take a register destination, so we need
            // to shuttle any memory destinations.
            asm::Statement::Binary(asm::Binary::Mul, operands) => {
                match self.allocate_binary(operands)? {
                    operands @ (operand::Binary::RI { .. }
                    | operand::Binary::RM { .. }
                    | operand::Binary::RR { .. }) => {
                        asm::Statement::Binary(asm::Binary::Mul, operands)
                    }
                    operand::Binary::MI {
                        destination,
                        source,
                    } => {
                        let shuttle = self.shuttle_unused.next()?;
                        self.statements.push(asm!((mov shuttle, destination)));
                        self.statements.push(asm!((imul shuttle, source)));
                        asm!((mov destination, shuttle))
                    }
                    operand::Binary::MR {
                        destination,
                        source,
                    } => {
                        let shuttle = self.shuttle_unused.next()?;
                        self.statements.push(asm!((mov shuttle, destination)));
                        self.statements.push(asm!((imul shuttle, source)));
                        asm!((mov destination, shuttle))
                    }
                }
            }

            asm::Statement::Binary(binary, operands) => {
                asm::Statement::Binary(*binary, self.allocate_binary(operands)?)
            }
            asm::Statement::Unary(unary, operand) => {
                asm::Statement::Unary(*unary, self.allocate_unary(operand)?)
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

        Some(())
    }

    fn allocate_binary(
        &mut self,
        binary: &operand::Binary<Temporary>,
    ) -> Option<operand::Binary<Register>> {
        let (destination, source) = match binary {
            operand::Binary::RI {
                destination,
                source,
            } => {
                return match self.allocate(destination) {
                    Or::L(register) => Some(operand::Binary::from((register, *source))),
                    Or::R(memory) => Some(operand::Binary::from((memory, *source))),
                }
            }
            operand::Binary::MI {
                destination,
                source,
            } => {
                return Some(operand::Binary::from((
                    self.allocate_memory(destination)?,
                    *source,
                )))
            }
            operand::Binary::MR {
                destination,
                source,
            } => (
                Or::R(self.allocate_memory(destination)?),
                self.allocate(source),
            ),
            operand::Binary::RM {
                destination,
                source,
            } => (
                self.allocate(destination),
                Or::R(self.allocate_memory(source)?),
            ),
            operand::Binary::RR {
                destination,
                source,
            } => (self.allocate(destination), self.allocate(source)),
        };

        let binary = match (destination, source) {
            (Or::L(destination), Or::L(source)) => operand::Binary::from((destination, source)),
            (Or::L(register), Or::R(memory)) => operand::Binary::from((register, memory)),
            (Or::R(memory), Or::L(register)) => operand::Binary::from((memory, register)),
            (Or::R(destination), Or::R(source)) => {
                let source = self.shuttle(asm::Binary::Mov, None, source)?;
                operand::Binary::from((destination, source))
            }
        };

        Some(binary)
    }

    fn allocate_unary(
        &mut self,
        unary: &operand::Unary<Temporary>,
    ) -> Option<operand::Unary<Register>> {
        let unary = match unary {
            operand::Unary::I(immediate) => operand::Unary::I(*immediate),
            operand::Unary::R(temporary) => match self.allocate(temporary) {
                Or::L(register) => operand::Unary::R(register),
                Or::R(memory) => operand::Unary::M(memory),
            },
            operand::Unary::M(memory) => operand::Unary::M(self.allocate_memory(memory)?),
        };

        Some(unary)
    }

    fn allocate_memory(
        &mut self,
        memory: &operand::Memory<Temporary>,
    ) -> Option<operand::Memory<Register>> {
        let (base, index, scale, offset) = match memory {
            Memory::O { offset } => return Some(Memory::O { offset: *offset }),
            Memory::B { base } => {
                return Some(Memory::B {
                    base: match self.allocate(base) {
                        Or::L(register) => register,
                        Or::R(memory) => self.shuttle(asm::Binary::Mov, None, memory)?,
                    },
                })
            }
            Memory::BO { base, offset } => {
                return Some(Memory::BO {
                    base: match self.allocate(base) {
                        Or::L(register) => register,
                        Or::R(memory) => self.shuttle(asm::Binary::Mov, None, memory)?,
                    },
                    offset: *offset,
                });
            }
            Memory::ISO {
                index,
                scale,
                offset,
            } => {
                return Some(Memory::ISO {
                    index: match self.allocate(index) {
                        Or::L(register) => register,
                        Or::R(memory) => self.shuttle(asm::Binary::Mov, None, memory)?,
                    },
                    scale: *scale,
                    offset: *offset,
                })
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
            (Or::L(base), Or::L(index), None, None) => return Some(Memory::BI { base, index }),
            (Or::L(base), Or::L(index), Some(scale), None) => {
                return Some(Memory::BIS { base, index, scale })
            }
            (Or::L(base), Or::L(index), None, Some(offset)) => {
                return Some(Memory::BIO {
                    base,
                    index,
                    offset,
                })
            }
            (Or::L(base), Or::L(index), Some(scale), Some(offset)) => {
                return Some(Memory::BISO {
                    base,
                    index,
                    scale,
                    offset,
                })
            }
            (base, index, _, _) => (base, index),
        };

        // Write destructively into the same shuttle register. Index first because
        // we might need to multiply it, and because addition is commutative.
        let shuttle = self.shuttle(asm::Binary::Mov, None, index)?;

        let shift = match scale {
            None | Some(Scale::_1) => None,
            Some(Scale::_2) => Some(1),
            Some(Scale::_4) => Some(2),
            Some(Scale::_8) => Some(3),
        };

        if let Some(shift) = shift {
            self.shuttle(asm::Binary::Shl, Some(shuttle), shift)?;
        }

        if let Some(offset) = offset {
            self.shuttle(asm::Binary::Add, Some(shuttle), offset)?;
        }

        Some(operand::Memory::B {
            base: self.shuttle(asm::Binary::Add, Some(shuttle), base)?,
        })
    }

    fn shuttle<S: Into<operand::Unary<Register>>>(
        &mut self,
        binary: asm::Binary,
        destination: Option<Register>,
        source: S,
    ) -> Option<Register> {
        let destination = match destination {
            Some(destination) => destination,
            None => self.shuttle_unused.next()?,
        };

        let operands = match source.into() {
            operand::Unary::R(register) => operand::Binary::from((destination, register)),
            operand::Unary::M(memory) => operand::Binary::from((destination, memory)),
            operand::Unary::I(immediate) => operand::Binary::from((destination, immediate)),
        };

        self.statements
            .push(asm::Statement::Binary(binary, operands));

        Some(destination)
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
fn rewrite_rbp(
    frame_pointer: abi::FramePointer,
    stack_size: i64,
    statement: &mut asm::Statement<Register>,
) {
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
            Immediate::Integer(offset) => {
                *offset += stack_size
                    + match frame_pointer {
                        abi::FramePointer::Keep => abi::WORD,
                        abi::FramePointer::Omit => 0,
                    };
            }
        }
    }
}
