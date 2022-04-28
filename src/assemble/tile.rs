#![allow(dead_code, unused_variables)]

use crate::data::asm;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;

struct Tiler {
    instructions: Vec<asm::Assembly<operand::Temporary>>,
    spilled: usize,
}

impl Tiler {
    fn tile_statement(&mut self, statement: &lir::Statement<lir::Fallthrough>) {
        match statement {
            lir::Statement::Jump(label) => {
                self.push(asm::Assembly::Unary(
                    asm::Unary::Jmp,
                    operand::One::I(operand::Immediate::Label(*label)),
                ));
            }
            lir::Statement::CJump {
                condition,
                r#true,
                r#false,
            } => todo!(),
            lir::Statement::Call(_, _, _) => todo!(),
            lir::Statement::Label(label) => self.push(asm::Assembly::Label(*label)),
            lir::Statement::Move {
                destination,
                source,
            } => {
                let binary = self.tile_binary(destination, source);
                self.push(asm::Assembly::Binary(asm::Binary::Mov, binary));
            }
            lir::Statement::Return => self.push(asm::Assembly::Nullary(asm::Nullary::Ret)),
        }
    }

    fn tile_expression(
        &mut self,
        expression: &lir::Expression,
    ) -> operand::One<operand::Temporary> {
        match expression {
            lir::Expression::Immediate(immediate) => operand::One::I(*immediate),
            lir::Expression::Temporary(temporary) => operand::One::R(*temporary),
            lir::Expression::Memory(address) => self.tile_memory(address),
            lir::Expression::Binary(_, _, _) => todo!(),
        }
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
    fn tile_binary(
        &mut self,
        destination: &lir::Expression,
        source: &lir::Expression,
    ) -> operand::Two<operand::Temporary> {
        match (
            self.tile_expression(destination),
            self.tile_expression(source),
        ) {
            (destination @ operand::One::I(_), operand::One::I(source)) => {
                let destination = self.shuttle(destination);
                operand::Two::RI {
                    destination,
                    source,
                }
            }
            (destination @ operand::One::I(_), operand::One::M(source)) => {
                let destination = self.shuttle(destination);
                operand::Two::RM {
                    destination,
                    source,
                }
            }

            (operand::One::M(destination), operand::One::I(source)) => operand::Two::MI {
                destination,
                source,
            },
            (operand::One::M(destination), source @ operand::One::M(_)) => operand::Two::MR {
                destination,
                source: self.shuttle(source),
            },

            (operand::One::R(destination), operand::One::I(source)) => operand::Two::RI {
                destination,
                source,
            },
            (operand::One::R(destination), operand::One::M(source)) => operand::Two::RM {
                destination,
                source,
            },

            (destination, source) => operand::Two::RR {
                destination: self.shuttle(destination),
                source: self.shuttle(source),
            },
        }
    }

    fn tile_memory(&mut self, address: &lir::Expression) -> operand::One<operand::Temporary> {
        let memory = match address {
            lir::Expression::Immediate(offset) => operand::Memory::O { offset: *offset },
            lir::Expression::Temporary(temporary) => operand::Memory::B { base: *temporary },
            lir::Expression::Memory(address) => operand::Memory::B {
                base: self.shuttle_memory(address),
            },
            lir::Expression::Binary(binary, left, right) => {
                use ir::Binary::Add;
                use ir::Binary::Mul;
                use ir::Binary::Sub;

                use lir::Expression::Binary;
                use lir::Expression::Immediate;
                use lir::Expression::Temporary;

                const EIGHT: &lir::Expression =
                    &lir::Expression::Immediate(operand::Immediate::Integer(8));

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
                                    (Mul, Temporary(index), EIGHT)
                                    | (Mul, EIGHT, Temporary(index)) => operand::Memory::BISO {
                                        base: *base,
                                        index: *index,
                                        scale: operand::Scale::_8,
                                        offset: *offset,
                                    },

                                    // [base + _index_ * 8 + offset]
                                    (Mul, tree, EIGHT) | (Mul, EIGHT, tree) => {
                                        operand::Memory::BISO {
                                            base: *base,
                                            index: self.shuttle_expression(tree),
                                            scale: operand::Scale::_8,
                                            offset: *offset,
                                        }
                                    }

                                    // [base + _index_ + offset]
                                    _ => operand::Memory::BIO {
                                        base: *base,
                                        index: self.shuttle_expression(tree),
                                        offset: *offset,
                                    },
                                }
                            }

                            // [base + index * 8]
                            (Mul, Temporary(index), EIGHT) | (Mul, EIGHT, Temporary(index)) => {
                                operand::Memory::BIS {
                                    base: *base,
                                    index: *index,
                                    scale: operand::Scale::_8,
                                }
                            }

                            // [base + index + offset]
                            (Add, Temporary(index), Immediate(offset))
                            | (Add, Immediate(offset), Temporary(index)) => operand::Memory::BIO {
                                base: *base,
                                index: *index,
                                offset: *offset,
                            },

                            // [base + _index_ * 8]
                            (Mul, tree, EIGHT) | (Mul, EIGHT, tree) => operand::Memory::BIS {
                                base: *base,
                                index: self.shuttle_expression(tree),
                                scale: operand::Scale::_8,
                            },

                            // [base + _index_ + offset]
                            (Add, tree, Immediate(offset)) | (Add, Immediate(offset), tree) => {
                                operand::Memory::BIO {
                                    base: *base,
                                    index: self.shuttle_expression(tree),
                                    offset: *offset,
                                }
                            }

                            // [base + _index_]
                            _ => operand::Memory::BI {
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
                                    (Mul, Temporary(index), EIGHT)
                                    | (Mul, EIGHT, Temporary(index)) => operand::Memory::BISO {
                                        base: *base,
                                        index: *index,
                                        scale: operand::Scale::_8,
                                        offset: *offset,
                                    },

                                    // [base + _index_ * 8 + offset]
                                    (Mul, tree, EIGHT) | (Mul, EIGHT, tree) => {
                                        operand::Memory::BISO {
                                            base: *base,
                                            index: self.shuttle_expression(tree),
                                            scale: operand::Scale::_8,
                                            offset: *offset,
                                        }
                                    }

                                    // [base + _index_ + offset
                                    _ => operand::Memory::BIO {
                                        base: *base,
                                        index: self.shuttle_expression(tree),
                                        offset: *offset,
                                    },
                                }
                            }

                            // [index * 8 + offset]
                            (Mul, Temporary(index), EIGHT) | (Mul, EIGHT, Temporary(index)) => {
                                operand::Memory::ISO {
                                    index: *index,
                                    scale: operand::Scale::_8,
                                    offset: *offset,
                                }
                            }

                            // [_index_ * 8 + offset]
                            (Mul, tree, EIGHT) | (Mul, EIGHT, tree) => operand::Memory::ISO {
                                index: self.shuttle_expression(tree),
                                scale: operand::Scale::_8,
                                offset: *offset,
                            },

                            // [_base_ + offset]
                            _ => operand::Memory::BO {
                                base: self.shuttle_expression(tree),
                                offset: *offset,
                            },
                        }
                    }

                    // [base + offset]
                    (Add, Temporary(base), Immediate(offset))
                    | (Add, Immediate(offset), Temporary(base)) => operand::Memory::BO {
                        base: *base,
                        offset: *offset,
                    },

                    // [base + -offset]
                    (Sub, Temporary(base), Immediate(operand::Immediate::Integer(offset))) => {
                        operand::Memory::BO {
                            base: *base,
                            offset: operand::Immediate::Integer(-*offset),
                        }
                    }

                    // [base + index]
                    (Add, Temporary(base), Temporary(index)) => operand::Memory::BI {
                        base: *base,
                        index: *index,
                    },

                    // [_base_ + offset]
                    (Add, Immediate(offset), tree) | (Add, tree, Immediate(offset)) => {
                        operand::Memory::BO {
                            base: self.shuttle_expression(tree),
                            offset: *offset,
                        }
                    }

                    // [base + _index_]
                    (Add, Temporary(base), tree) | (Add, tree, Temporary(base)) => {
                        operand::Memory::BI {
                            base: *base,
                            index: self.shuttle_expression(tree),
                        }
                    }

                    // [_base_]
                    _ => operand::Memory::B {
                        base: self.shuttle_expression(address),
                    },
                }
            }
        };

        operand::One::M(memory)
    }

    fn shuttle_memory(&mut self, address: &lir::Expression) -> operand::Temporary {
        let address = self.tile_memory(address);
        self.shuttle(address)
    }

    fn shuttle_expression(&mut self, expression: &lir::Expression) -> operand::Temporary {
        let expression = self.tile_expression(expression);
        self.shuttle(expression)
    }

    fn shuttle(&mut self, operand: operand::One<operand::Temporary>) -> operand::Temporary {
        match operand {
            operand::One::R(temporary) => temporary,
            operand::One::I(source) => {
                let destination = operand::Temporary::fresh("shuttle");
                self.push(asm::Assembly::Binary(
                    asm::Binary::Mov,
                    operand::Two::RI {
                        destination,
                        source,
                    },
                ));
                destination
            }
            operand::One::M(source) => {
                let destination = operand::Temporary::fresh("shuttle");
                self.push(asm::Assembly::Binary(
                    asm::Binary::Mov,
                    operand::Two::RM {
                        destination,
                        source,
                    },
                ));
                destination
            }
        }
    }

    fn push(&mut self, instruction: asm::Assembly<operand::Temporary>) {
        self.instructions.push(instruction);
    }
}
