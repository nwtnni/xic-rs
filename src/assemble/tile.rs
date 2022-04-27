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
        todo!()
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

                use operand::Immediate::Integer;

                match (binary, &**left, &**right) {
                    // Ternary addressing modes
                    (Add, Temporary(base), tree @ Binary(binary, left, right))
                    | (Add, tree @ Binary(binary, left, right), Temporary(base)) => {
                        match (binary, &**left, &**right) {
                            (Mul, Temporary(index), Immediate(operand::Immediate::Integer(8)))
                            | (Mul, Immediate(operand::Immediate::Integer(8)), Temporary(index)) => {
                                operand::Memory::BIS {
                                    base: *base,
                                    index: *index,
                                    scale: operand::Scale::_8,
                                }
                            }
                            (Mul, tree, Immediate(operand::Immediate::Integer(8)))
                            | (Mul, Immediate(operand::Immediate::Integer(8)), tree) => {
                                operand::Memory::BIS {
                                    base: *base,
                                    index: self.shuttle_expression(tree),
                                    scale: operand::Scale::_8,
                                }
                            }

                            (Add, Temporary(index), Immediate(offset))
                            | (Add, Immediate(offset), Temporary(index)) => operand::Memory::BIO {
                                base: *base,
                                index: *index,
                                offset: *offset,
                            },
                            (Add, tree, Immediate(offset)) | (Add, Immediate(offset), tree) => {
                                operand::Memory::BIO {
                                    base: *base,
                                    index: self.shuttle_expression(tree),
                                    offset: *offset,
                                }
                            }

                            _ => operand::Memory::BI {
                                base: *base,
                                index: self.shuttle_expression(tree),
                            },
                        }
                    }

                    (Add, Immediate(offset), tree @ Binary(binary, left, right))
                    | (Add, tree @ Binary(binary, left, right), Immediate(offset)) => {
                        match (binary, &**left, &**right) {
                            (Mul, Temporary(index), Immediate(operand::Immediate::Integer(8)))
                            | (Mul, Immediate(operand::Immediate::Integer(8)), Temporary(index)) => {
                                operand::Memory::ISO {
                                    index: *index,
                                    scale: operand::Scale::_8,
                                    offset: *offset,
                                }
                            }
                            (Mul, tree, Immediate(operand::Immediate::Integer(8)))
                            | (Mul, Immediate(operand::Immediate::Integer(8)), tree) => {
                                operand::Memory::ISO {
                                    index: self.shuttle_expression(tree),
                                    scale: operand::Scale::_8,
                                    offset: *offset,
                                }
                            }

                            _ => operand::Memory::BO {
                                base: self.shuttle_expression(tree),
                                offset: *offset,
                            },
                        }
                    }

                    // Binary addressing modes
                    (Add, Temporary(base), Immediate(offset))
                    | (Add, Immediate(offset), Temporary(base)) => operand::Memory::BO {
                        base: *base,
                        offset: *offset,
                    },
                    (Sub, Temporary(base), Immediate(Integer(offset))) => operand::Memory::BO {
                        base: *base,
                        offset: Integer(-*offset),
                    },

                    (Add, Temporary(base), Temporary(index)) => operand::Memory::BI {
                        base: *base,
                        index: *index,
                    },

                    // Default binary addressing modes
                    (Add, Temporary(base), tree) | (Add, tree, Temporary(base)) => {
                        operand::Memory::BI {
                            base: *base,
                            index: self.shuttle_expression(tree),
                        }
                    }
                    (Add, Immediate(offset), tree) | (Add, tree, Immediate(offset)) => {
                        operand::Memory::BO {
                            base: self.shuttle_expression(tree),
                            offset: *offset,
                        }
                    }

                    // Default unary addressing mode
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
