use crate::abi;
use crate::asm;
use crate::data::asm;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::util;
use crate::util::Or;

struct Tiler {
    statements: Vec<asm::Statement<Temporary>>,
    caller_returns: Option<Temporary>,
    callee_arguments: usize,
}

enum Mutate {
    Yes,
    No,
}

pub fn tile(
    frame_pointer: abi::FramePointer,
    function: &lir::Function<lir::Fallthrough>,
) -> asm::Function<Temporary> {
    log::info!(
        "[{}] Tiling {}...",
        std::any::type_name::<lir::Function<lir::Fallthrough>>(),
        function.name,
    );
    util::time!(
        "[{}] Done tiling {}",
        std::any::type_name::<lir::Function<lir::Fallthrough>>(),
        function.name,
    );

    let caller_returns = match function.returns > abi::RETURN.len() {
        true => Some(Temporary::fresh("return")),
        false => None,
    };

    let mut tiler = Tiler {
        statements: Vec::new(),
        caller_returns,
        callee_arguments: function.callee_arguments().unwrap_or(0),
    };

    assert!(matches!(
        function.statements.first(),
        Some(lir::Statement::Label(label)) if *label == function.enter,
    ));

    // Preserve invariant that `enter` label is the first statement
    tiler.tile_statement(function.statements.first().unwrap());

    if frame_pointer == abi::FramePointer::Keep {
        tiler.push(asm!((push rbp)));
        tiler.push(asm!((mov rbp, rsp)));
    }

    let callee_saved = abi::CALLEE_SAVED
        .iter()
        .copied()
        .filter(|register| *register != Register::rsp())
        // If omitting frame pointer, then treat it as a regular callee-saved register
        .filter(|register| *register != Register::Rbp || frame_pointer == abi::FramePointer::Omit)
        .map(|register| {
            let temporary = Temporary::fresh("save");
            let register = Temporary::Register(register);
            tiler.push(asm!((mov temporary, register)));
            (temporary, register)
        })
        .collect::<Vec<_>>();

    tiler.tile_arguments(&function.arguments);

    function
        .statements
        .iter()
        .skip(1)
        .for_each(|statement| tiler.tile_statement(statement));

    for (temporary, register) in callee_saved {
        tiler.push(asm!((mov register, temporary)));
    }

    if frame_pointer == abi::FramePointer::Keep {
        tiler.push(asm!((pop rbp)));
    }

    tiler.push(asm::Statement::Nullary(asm::Nullary::Ret(function.returns)));

    asm::Function {
        name: function.name,
        statements: tiler.statements,
        arguments: function.arguments.len(),
        returns: function.returns,
        linkage: function.linkage,
        enter: function.enter,
        exit: function.exit,
    }
}

impl Tiler {
    fn tile_arguments(&mut self, arguments: &[Temporary]) {
        let offset = match self.caller_returns {
            None => 0,
            Some(temporary) => {
                self.tile_binary(asm::Binary::Mov, temporary, abi::read_argument(0));
                1
            }
        };

        for (index, argument) in arguments.iter().copied().enumerate() {
            self.tile_binary(
                asm::Binary::Mov,
                argument,
                // Note: we add this offset here and not above to keep the extra argument
                // abstracted away from the IR level.
                abi::read_argument(index + offset),
            );
        }
    }

    fn tile_statement(&mut self, statement: &lir::Statement<lir::Fallthrough>) {
        match statement {
            lir::Statement::Label(label) => self.push(asm::Statement::Label(*label)),
            lir::Statement::Return(returns) => {
                for (index, r#return) in returns.iter().enumerate() {
                    self.tile_binary(
                        asm::Binary::Mov,
                        abi::write_return(self.caller_returns, index),
                        r#return,
                    );
                }

                // CFG construction guarantees that (1) an IR return is immediately
                // followed by a jump to the exit label, and (2) the exit block is
                // at the end. Then we omit the `ret` statement here, in favor of
                // placing a single `ret` at the end of the function epilogue:
                //
                // ```
                // self.push(asm::Statement::Nullary(asm::Nullary::Ret(function.returns)));
                // ```
            }
            &lir::Statement::Jump(label) => {
                self.push(asm!((jmp label)));
            }
            &lir::Statement::CJump {
                condition,
                ref left,
                ref right,
                r#true,
                r#false: lir::Fallthrough,
            } => {
                self.tile_binary(asm::Binary::Cmp, left, right);
                self.push(asm!((jcc asm::Condition::from(condition), r#true)));
            }
            // Special case: 64-bit immediate can only be passed to `mov r64, i64`.
            &lir::Statement::Move {
                destination: lir::Expression::Temporary(temporary),
                source: lir::Expression::Immediate(Immediate::Integer(integer)),
            } => {
                self.push(asm!((mov temporary, integer)));
            }
            lir::Statement::Move {
                destination,
                source,
            } => match source {
                lir::Expression::Binary(binary, left, right) => {
                    let (mutate, source) =
                        self.tile_binary_expression(binary, left, right, Some(destination));
                    match mutate {
                        Mutate::Yes => (),
                        Mutate::No => self.tile_binary(asm::Binary::Mov, destination, source),
                    }
                }
                source => self.tile_binary(asm::Binary::Mov, destination, source),
            },
            lir::Statement::Call(function, arguments, returns) => {
                let offset = if returns.len() > 2 {
                    self.tile_binary(
                        asm::Binary::Lea,
                        abi::write_argument(0),
                        abi::read_return(self.callee_arguments, 2),
                    );
                    1
                } else {
                    0
                };

                for (index, argument) in arguments.iter().enumerate() {
                    self.tile_binary(
                        asm::Binary::Mov,
                        abi::write_argument(index + offset),
                        argument,
                    );
                }

                let function = self.tile_expression(function);
                let arguments = arguments.len() + offset;

                #[rustfmt::skip]
                self.push(asm!((call<arguments, (returns.len())> function)));

                for (index, r#return) in returns.iter().copied().enumerate() {
                    self.tile_binary(
                        asm::Binary::Mov,
                        r#return,
                        abi::read_return(self.callee_arguments, index),
                    );
                }
            }
        }
    }

    fn tile_expression(&mut self, expression: &lir::Expression) -> operand::Unary<Temporary> {
        let (binary, left, right) = match expression {
            lir::Expression::Immediate(immediate) => {
                // Only `mov r64, i64` statements can use 64-bit immediates (handled above).
                return match immediate.is_64_bit() {
                    false => operand::Unary::I(*immediate),
                    true => operand::Unary::R(self.shuttle(operand::Unary::I(*immediate))),
                };
            }
            lir::Expression::Temporary(temporary) => return operand::Unary::R(*temporary),
            lir::Expression::Memory(address) => return self.tile_memory(address),
            lir::Expression::Binary(binary, left, right) => (binary, &**left, &**right),
        };

        let (_, destination) = self.tile_binary_expression(binary, left, right, None);
        destination
    }

    // Binary expressions can reuse the same destination for destructive assembly statements.
    fn tile_binary_expression(
        &mut self,
        binary: &ir::Binary,
        left: &lir::Expression,
        right: &lir::Expression,
        destination: Option<&lir::Expression>,
    ) -> (Mutate, operand::Unary<Temporary>) {
        // Special-case unary operator
        if let (ir::Binary::Sub, lir::Expression::Immediate(Immediate::Integer(0))) = (binary, left)
        {
            let (mutate, destination) = match right {
                lir::Expression::Binary(binary, left, right) => {
                    self.tile_binary_expression(binary, left, right, destination)
                }
                expression if Some(expression) == destination => {
                    (Mutate::Yes, self.tile_expression(expression))
                }
                expression => {
                    let fresh = Temporary::fresh("tile");
                    self.tile_binary(asm::Binary::Mov, fresh, expression);
                    (Mutate::No, operand::Unary::R(fresh))
                }
            };

            self.tile_unary(asm::Unary::Neg, destination);
            return (mutate, destination);
        }

        let (mutate, destination) = match left {
            lir::Expression::Binary(binary, left, right) => {
                self.tile_binary_expression(binary, left, right, destination)
            }
            expression if Some(expression) == destination => {
                (Mutate::Yes, self.tile_expression(expression))
            }
            expression => {
                let fresh = Temporary::fresh("tile");
                self.tile_binary(asm::Binary::Mov, fresh, expression);
                (Mutate::No, operand::Unary::R(fresh))
            }
        };

        match binary {
            ir::Binary::Add
            | ir::Binary::Sub
            | ir::Binary::And
            | ir::Binary::Mul
            | ir::Binary::Or
            | ir::Binary::Xor => {
                self.tile_binary(asm::Binary::from(*binary), destination, right);
            }
            ir::Binary::Hul | ir::Binary::Div | ir::Binary::Mod => {
                let (cqo, unary, register) = match binary {
                    ir::Binary::Hul => (false, asm::Unary::Hul, Register::Rdx),
                    ir::Binary::Div => (true, asm::Unary::Div, Register::Rax),
                    ir::Binary::Mod => (true, asm::Unary::Mod, Register::Rdx),
                    _ => unreachable!(),
                };

                // Tile source before moving `destination` into `rax`, because
                // it could clobber `rax` or `rdx`.
                let source = self.tile_expression(right);
                let source = self.shuttle(source);

                self.tile_binary(asm::Binary::Mov, Register::Rax, destination);
                if cqo {
                    self.push(asm!((cqo)));
                }
                self.tile_unary(unary, source);
                self.tile_binary(asm::Binary::Mov, destination, register);
            }
        }

        (mutate, destination)
    }

    // ```text
    //               source
    //               I R M
    //             I d d d
    // destination R _ _ _
    //             M _ _ s
    //
    // d: shuttle destination
    // s: shuttle source
    // _: no shuttle necessary
    // ```
    fn tile_binary<'a>(
        &mut self,
        binary: asm::Binary,
        destination: impl Into<Or<&'a lir::Expression, operand::Unary<Temporary>>>,
        source: impl Into<Or<&'a lir::Expression, operand::Unary<Temporary>>>,
    ) {
        let destination = match destination.into() {
            Or::L(expression) => self.tile_expression(expression),
            Or::R(destination) => destination,
        };

        let source = match source.into() {
            Or::L(expression) => self.tile_expression(expression),
            Or::R(source) => source,
        };

        use operand::Binary;
        use operand::Unary;

        let operands = match (destination, source) {
            (destination @ Unary::I(_), Unary::I(source)) => Binary::RI {
                destination: self.shuttle(destination),
                source,
            },
            (destination @ Unary::I(_), Unary::R(source)) => Binary::RR {
                destination: self.shuttle(destination),
                source,
            },
            (destination @ Unary::I(_), Unary::M(source)) => Binary::RM {
                destination: self.shuttle(destination),
                source,
            },

            (Unary::M(destination), Unary::I(source)) => Binary::MI {
                destination,
                source,
            },
            (Unary::M(destination), Unary::R(source)) => Binary::MR {
                destination,
                source,
            },
            (Unary::M(destination), source @ Unary::M(_)) => Binary::MR {
                destination,
                source: self.shuttle(source),
            },

            (Unary::R(destination), Unary::I(source)) => Binary::RI {
                destination,
                source,
            },
            (Unary::R(destination), Unary::R(source)) => Binary::RR {
                destination,
                source,
            },
            (Unary::R(destination), Unary::M(source)) => Binary::RM {
                destination,
                source,
            },
        };

        self.push(asm::Statement::Binary(binary, operands));
    }

    fn tile_unary<'a>(
        &mut self,
        unary: asm::Unary,
        destination: impl Into<Or<&'a lir::Expression, operand::Unary<Temporary>>>,
    ) -> operand::Unary<Temporary> {
        let destination = match destination.into() {
            Or::L(expression) => self.tile_expression(expression),
            Or::R(operand) => operand,
        };

        self.push(asm::Statement::Unary(unary, destination));
        destination
    }

    fn tile_memory(&mut self, address: &lir::Expression) -> operand::Unary<Temporary> {
        let memory = match address {
            lir::Expression::Immediate(offset) => Memory::O { offset: *offset },
            lir::Expression::Temporary(temporary) => Memory::B { base: *temporary },
            lir::Expression::Memory(address) => Memory::B {
                base: self.shuttle_memory(address),
            },
            lir::Expression::Binary(binary, left, right) => {
                use ir::Binary::Add;
                use ir::Binary::Mul;
                use ir::Binary::Sub;

                use lir::Expression::Binary;
                use lir::Expression::Immediate;
                use lir::Expression::Temporary;

                match (binary, &**left, &**right) {
                    // [base + ...]
                    (Add, Temporary(base), tree @ Binary(binary, left, right))
                    | (Add, tree @ Binary(binary, left, right), Temporary(base)) => {
                        match (binary, &**left, &**right) {
                            // [base + ... + offset]
                            (Add, Immediate(offset), tree @ Binary(binary, left, right))
                            | (Add, tree @ Binary(binary, left, right), Immediate(offset)) => {
                                match (binary, &**left, &**right) {
                                    // [base + index * 8 + offset]
                                    (Mul, Temporary(index), &lir::EIGHT)
                                    | (Mul, &lir::EIGHT, Temporary(index)) => Memory::BISO {
                                        base: *base,
                                        index: *index,
                                        scale: operand::Scale::_8,
                                        offset: *offset,
                                    },

                                    // [base + _index_ * 8 + offset]
                                    (Mul, tree, &lir::EIGHT) | (Mul, &lir::EIGHT, tree) => {
                                        Memory::BISO {
                                            base: *base,
                                            index: self.shuttle_expression(tree),
                                            scale: operand::Scale::_8,
                                            offset: *offset,
                                        }
                                    }

                                    // [base + _index_ + offset]
                                    _ => Memory::BIO {
                                        base: *base,
                                        index: self.shuttle_expression(tree),
                                        offset: *offset,
                                    },
                                }
                            }

                            // [base + index * 8]
                            (Mul, Temporary(index), &lir::EIGHT)
                            | (Mul, &lir::EIGHT, Temporary(index)) => Memory::BIS {
                                base: *base,
                                index: *index,
                                scale: operand::Scale::_8,
                            },

                            // [base + index + offset]
                            (Add, Temporary(index), Immediate(offset))
                            | (Add, Immediate(offset), Temporary(index)) => Memory::BIO {
                                base: *base,
                                index: *index,
                                offset: *offset,
                            },

                            // [base + _index_ * 8]
                            (Mul, tree, &lir::EIGHT) | (Mul, &lir::EIGHT, tree) => Memory::BIS {
                                base: *base,
                                index: self.shuttle_expression(tree),
                                scale: operand::Scale::_8,
                            },

                            // [base + _index_ + offset]
                            (Add, tree, Immediate(offset)) | (Add, Immediate(offset), tree) => {
                                Memory::BIO {
                                    base: *base,
                                    index: self.shuttle_expression(tree),
                                    offset: *offset,
                                }
                            }

                            // [base + _index_]
                            _ => Memory::BI {
                                base: *base,
                                index: self.shuttle_expression(tree),
                            },
                        }
                    }

                    // [... + offset]
                    (Add, Immediate(offset), tree @ Binary(binary, left, right))
                    | (Add, tree @ Binary(binary, left, right), Immediate(offset)) => {
                        match (binary, &**left, &**right) {
                            // [base + ... + offset]
                            (Add, Temporary(base), tree @ Binary(binary, left, right))
                            | (Add, tree @ Binary(binary, left, right), Temporary(base)) => {
                                match (binary, &**left, &**right) {
                                    // [base + index * 8 + offset]
                                    (Mul, Temporary(index), &lir::EIGHT)
                                    | (Mul, &lir::EIGHT, Temporary(index)) => Memory::BISO {
                                        base: *base,
                                        index: *index,
                                        scale: operand::Scale::_8,
                                        offset: *offset,
                                    },

                                    // [base + _index_ * 8 + offset]
                                    (Mul, tree, &lir::EIGHT) | (Mul, &lir::EIGHT, tree) => {
                                        Memory::BISO {
                                            base: *base,
                                            index: self.shuttle_expression(tree),
                                            scale: operand::Scale::_8,
                                            offset: *offset,
                                        }
                                    }

                                    // [base + _index_ + offset
                                    _ => Memory::BIO {
                                        base: *base,
                                        index: self.shuttle_expression(tree),
                                        offset: *offset,
                                    },
                                }
                            }

                            // [index * 8 + offset]
                            (Mul, Temporary(index), &lir::EIGHT)
                            | (Mul, &lir::EIGHT, Temporary(index)) => Memory::ISO {
                                index: *index,
                                scale: operand::Scale::_8,
                                offset: *offset,
                            },

                            // [_index_ * 8 + offset]
                            (Mul, tree, &lir::EIGHT) | (Mul, &lir::EIGHT, tree) => Memory::ISO {
                                index: self.shuttle_expression(tree),
                                scale: operand::Scale::_8,
                                offset: *offset,
                            },

                            // [_base_ + offset]
                            _ => Memory::BO {
                                base: self.shuttle_expression(tree),
                                offset: *offset,
                            },
                        }
                    }

                    // [base + offset]
                    (Add, Temporary(base), Immediate(offset))
                    | (Add, Immediate(offset), Temporary(base)) => Memory::BO {
                        base: *base,
                        offset: *offset,
                    },

                    // [base + -offset]
                    (Sub, Temporary(base), Immediate(operand::Immediate::Integer(offset))) => {
                        Memory::BO {
                            base: *base,
                            offset: operand::Immediate::Integer(-*offset),
                        }
                    }

                    // [base + index]
                    (Add, Temporary(base), Temporary(index)) => Memory::BI {
                        base: *base,
                        index: *index,
                    },

                    // [_base_ + offset]
                    (Add, Immediate(offset), tree) | (Add, tree, Immediate(offset)) => Memory::BO {
                        base: self.shuttle_expression(tree),
                        offset: *offset,
                    },

                    // [base + _index_]
                    (Add, Temporary(base), tree) | (Add, tree, Temporary(base)) => Memory::BI {
                        base: *base,
                        index: self.shuttle_expression(tree),
                    },

                    // [_base_]
                    _ => Memory::B {
                        base: self.shuttle_expression(address),
                    },
                }
            }
        };

        operand::Unary::M(memory)
    }

    fn shuttle_memory(&mut self, address: &lir::Expression) -> Temporary {
        let address = self.tile_memory(address);
        self.shuttle(address)
    }

    fn shuttle_expression(&mut self, expression: &lir::Expression) -> Temporary {
        let expression = self.tile_expression(expression);
        self.shuttle(expression)
    }

    fn shuttle(&mut self, operand: operand::Unary<Temporary>) -> Temporary {
        match operand {
            operand::Unary::R(temporary) => temporary,
            operand::Unary::I(source) => {
                let destination = Temporary::fresh("shuttle");
                self.push(asm!((mov destination, source)));
                destination
            }
            operand::Unary::M(source) => {
                let destination = Temporary::fresh("shuttle");
                self.push(asm!((mov destination, source)));
                destination
            }
        }
    }

    fn push(&mut self, statement: asm::Statement<Temporary>) {
        self.statements.push(statement);
    }
}

impl<'a> From<&'a lir::Expression> for Or<&'a lir::Expression, operand::Unary<Temporary>> {
    fn from(expression: &'a lir::Expression) -> Self {
        Or::L(expression)
    }
}

impl<T: Into<operand::Unary<Temporary>>> From<T>
    for Or<&lir::Expression, operand::Unary<Temporary>>
{
    fn from(temporary: T) -> Self {
        Or::R(temporary.into())
    }
}
