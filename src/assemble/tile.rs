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
            lir::Expression::Integer(integer) => {
                operand::One::I(operand::Immediate::Constant(*integer))
            }
            lir::Expression::Label(label) => operand::One::I(operand::Immediate::Label(*label)),
            lir::Expression::Temporary(temporary) => operand::One::R(*temporary),
            lir::Expression::Memory(address) => self.tile_memory(address),
            lir::Expression::Binary(_, _, _) => todo!(),
        }
    }

    fn tile_memory(&mut self, address: &lir::Expression) -> operand::One<operand::Temporary> {
        match address {
            lir::Expression::Integer(offset) => operand::One::M(operand::Memory::O {
                offset: operand::Immediate::Constant(*offset),
            }),
            lir::Expression::Label(label) => operand::One::M(operand::Memory::O {
                offset: operand::Immediate::Label(*label),
            }),
            lir::Expression::Temporary(temporary) => {
                operand::One::M(operand::Memory::B { base: *temporary })
            }
            lir::Expression::Memory(address) => {
                let address = self.tile_memory(address);
                let shuttle = self.shuttle(address);
                operand::One::M(operand::Memory::B { base: shuttle })
            }
            lir::Expression::Binary(binary, left, right) => match (binary, &**left, &**right) {
                (
                    ir::Binary::Add,
                    lir::Expression::Temporary(base),
                    lir::Expression::Integer(offset),
                ) => operand::One::M(operand::Memory::BO {
                    base: *base,
                    offset: operand::Immediate::Constant(*offset),
                }),
                _ => {
                    let address = self.tile_expression(address);
                    let shuttle = self.shuttle(address);
                    operand::One::M(operand::Memory::B { base: shuttle })
                }
            },
        }
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
