#![allow(dead_code, unused_variables)]

use crate::abi;
use crate::data::asm;
use crate::data::asm::Assembly;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Temporary;

struct Tiler {
    instructions: Vec<Assembly<Temporary>>,
    caller_returns: Option<Temporary>,
    callee_arguments: usize,
    callee_returns: usize,
}

impl Tiler {
    fn tile_statement(&mut self, statement: &lir::Statement<lir::Fallthrough>) {
        match statement {
            lir::Statement::Label(label) => self.push(Assembly::Label(*label)),
            lir::Statement::Return(returns) => {
                for (index, r#return) in returns.iter().enumerate() {
                    let destination = abi::write_return(self.caller_returns, index);
                    let source = self.tile_expression(r#return);
                    let operands = self.shuttle_binary(destination, source);
                    self.push(Assembly::Binary(asm::Binary::Mov, operands));
                }
                self.push(Assembly::Nullary(asm::Nullary::Ret));
            }
            lir::Statement::Jump(label) => {
                self.push(Assembly::Unary(
                    asm::Unary::Jmp,
                    operand::Unary::from(*label),
                ));
            }
            lir::Statement::CJump {
                condition,
                left,
                right,
                r#true,
                r#false: lir::Fallthrough,
            } => {
                let operands = self.tile_binary(left, right);
                self.push(Assembly::Binary(asm::Binary::Cmp, operands));
                self.push(Assembly::Unary(
                    asm::Unary::Jcc(asm::Condition::from(*condition)),
                    operand::Unary::from(*r#true),
                ));
            }
            lir::Statement::Move {
                destination,
                source,
            } => {
                use ir::Binary::Add;
                use ir::Binary::And;
                use ir::Binary::Or;
                use ir::Binary::Sub;
                use ir::Binary::Xor;

                let (binary, source) = match source {
                    lir::Expression::Binary(binary @ (Add | And | Or | Xor), left, right)
                        if &**left == destination =>
                    {
                        (asm::Binary::from(*binary), &**right)
                    }

                    lir::Expression::Binary(binary @ (Add | And | Or | Xor), left, right)
                        if &**right == destination =>
                    {
                        (asm::Binary::from(*binary), &**left)
                    }

                    lir::Expression::Binary(Sub, left, right) if &**left == destination => {
                        (asm::Binary::Sub, &**right)
                    }

                    lir::Expression::Binary(Sub, left, right)
                        if **left == lir::Expression::from(0) && &**right == destination =>
                    {
                        let operand = self.tile_expression(destination);
                        self.push(Assembly::Unary(asm::Unary::Neg, operand));
                        return;
                    }

                    _ => (asm::Binary::Mov, source),
                };

                let operands = self.tile_binary(destination, source);
                self.push(Assembly::Binary(binary, operands));
            }
            lir::Statement::Call(_, _, _) => todo!(),
        }
    }

    fn tile_expression(&mut self, expression: &lir::Expression) -> operand::Unary<Temporary> {
        let (binary, destination, source) = match expression {
            lir::Expression::Argument(index) => {
                // Adjust for implicit 0th argument (multiple return temporary)
                return abi::read_argument(*index + self.caller_returns.map(|_| 1).unwrap_or(0));
            }
            lir::Expression::Return(index) => {
                return abi::read_return(self.callee_arguments, *index)
            }
            lir::Expression::Immediate(immediate) => return operand::Unary::I(*immediate),
            lir::Expression::Temporary(temporary) => return operand::Unary::R(*temporary),
            lir::Expression::Memory(address) => return self.tile_memory(address),
            lir::Expression::Binary(binary, left, right) => (binary, &**left, &**right),
        };

        // Special-case unary operator
        if let (ir::Binary::Sub, lir::Expression::Immediate(Immediate::Integer(0))) =
            (binary, destination)
        {
            let operand = match self.tile_expression(source) {
                operand @ (operand::Unary::I(_) | operand::Unary::M(_)) => self.shuttle(operand),
                operand::Unary::R(source) => {
                    let destination = Temporary::fresh("tile");
                    self.push(Assembly::Binary(
                        asm::Binary::Mov,
                        operand::Binary::RR {
                            destination,
                            source,
                        },
                    ));
                    destination
                }
            };

            self.push(Assembly::Unary(asm::Unary::Neg, operand::Unary::R(operand)));

            return operand::Unary::R(operand);
        }

        match binary {
            ir::Binary::Add
            | ir::Binary::Sub
            | ir::Binary::And
            | ir::Binary::Or
            | ir::Binary::Xor => {
                let fresh = Temporary::fresh("tile");

                let r#move = Assembly::Binary(
                    asm::Binary::Mov,
                    self.tile_binary(&lir::Expression::Temporary(fresh), destination),
                );
                let binary = Assembly::Binary(
                    asm::Binary::from(*binary),
                    self.tile_binary(&lir::Expression::Temporary(fresh), source),
                );

                self.push(r#move);
                self.push(binary);

                operand::Unary::R(fresh)
            }
            ir::Binary::Mul | ir::Binary::Hul | ir::Binary::Div | ir::Binary::Mod => {
                use asm::Division::Quotient;
                use asm::Division::Remainder;

                let (cqo, unary, register) = match binary {
                    ir::Binary::Mul => (false, asm::Unary::Mul, Register::Rax),
                    ir::Binary::Hul => (false, asm::Unary::Mul, Register::Rdx),
                    ir::Binary::Div => (true, asm::Unary::Div(Quotient), Register::Rax),
                    ir::Binary::Mod => (true, asm::Unary::Div(Remainder), Register::Rdx),
                    _ => unreachable!(),
                };

                let destination = Assembly::Binary(
                    asm::Binary::Mov,
                    self.tile_binary(&lir::Expression::from(Register::Rax), destination),
                );

                let source = Assembly::Unary(
                    unary,
                    match self.tile_expression(source) {
                        source @ (operand::Unary::R(_) | operand::Unary::M(_)) => source,
                        source @ operand::Unary::I(_) => operand::Unary::R(self.shuttle(source)),
                    },
                );

                let fresh = Temporary::fresh("tile");

                self.push(destination);
                if cqo {
                    self.push(Assembly::Nullary(asm::Nullary::Cqo));
                }
                self.push(source);
                self.push(Assembly::Binary(
                    asm::Binary::Mov,
                    operand::Binary::RR {
                        destination: fresh,
                        source: Temporary::Register(register),
                    },
                ));

                operand::Unary::R(fresh)
            }
        }
    }

    fn tile_binary(
        &mut self,
        destination: &lir::Expression,
        source: &lir::Expression,
    ) -> operand::Binary<Temporary> {
        let destination = self.tile_expression(destination);
        let source = self.tile_expression(source);
        self.shuttle_binary(destination, source)
    }

    fn tile_memory(&mut self, address: &lir::Expression) -> operand::Unary<Temporary> {
        let memory = match address {
            lir::Expression::Argument(index) => Memory::B {
                base: self.shuttle(abi::read_argument(*index)),
            },
            lir::Expression::Return(index) => Memory::B {
                base: self.shuttle(abi::read_return(self.callee_arguments, *index)),
            },
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
    fn shuttle_binary(
        &mut self,
        destination: operand::Unary<Temporary>,
        source: operand::Unary<Temporary>,
    ) -> operand::Binary<Temporary> {
        use operand::Binary;
        use operand::Unary;

        match (destination, source) {
            (destination @ Unary::I(_), Unary::I(source)) => {
                let destination = self.shuttle(destination);
                Binary::RI {
                    destination,
                    source,
                }
            }
            (destination @ Unary::I(_), Unary::M(source)) => {
                let destination = self.shuttle(destination);
                Binary::RM {
                    destination,
                    source,
                }
            }

            (Unary::M(destination), Unary::I(source)) => Binary::MI {
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
            (Unary::R(destination), Unary::M(source)) => Binary::RM {
                destination,
                source,
            },

            (destination, source) => Binary::RR {
                destination: self.shuttle(destination),
                source: self.shuttle(source),
            },
        }
    }

    fn shuttle(&mut self, operand: operand::Unary<Temporary>) -> Temporary {
        match operand {
            operand::Unary::R(temporary) => temporary,
            operand::Unary::I(source) => {
                let destination = Temporary::fresh("shuttle");
                self.push(Assembly::Binary(
                    asm::Binary::Mov,
                    operand::Binary::RI {
                        destination,
                        source,
                    },
                ));
                destination
            }
            operand::Unary::M(source) => {
                let destination = Temporary::fresh("shuttle");
                self.push(Assembly::Binary(
                    asm::Binary::Mov,
                    operand::Binary::RM {
                        destination,
                        source,
                    },
                ));
                destination
            }
        }
    }

    fn push(&mut self, instruction: Assembly<Temporary>) {
        self.instructions.push(instruction);
    }
}
