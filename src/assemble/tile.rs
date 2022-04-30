use std::cmp;

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
use crate::util::Or;

struct Tiler {
    instructions: Vec<Assembly<Temporary>>,
    caller_returns: Option<Temporary>,
    callee_arguments: usize,
    #[allow(dead_code)]
    callee_returns: usize,
}

enum Mutate {
    Yes,
    No,
}

pub fn tile_unit(unit: &lir::Unit<lir::Fallthrough>) -> asm::Unit<Temporary> {
    unit.map(tile_function)
}

fn tile_function(function: &lir::Function<lir::Fallthrough>) -> asm::Function<Temporary> {
    let caller_returns = match function.returns {
        0 | 1 | 2 => None,
        _ => Some(Temporary::fresh("overflow")),
    };

    let (callee_arguments, callee_returns) = function
        .statements
        .iter()
        .filter_map(|statement| match statement {
            lir::Statement::Call(_, arguments, returns) => Some((arguments.len(), *returns)),
            _ => None,
        })
        .fold(
            (0, 0),
            |(callee_arguments, callee_returns), (arguments, returns)| {
                (
                    cmp::max(callee_arguments, arguments),
                    cmp::max(callee_returns, returns),
                )
            },
        );

    let mut tiler = Tiler {
        instructions: Vec::new(),
        caller_returns,
        callee_arguments,
        callee_returns,
    };

    let callee_saved = abi::CALLEE_SAVED
        .iter()
        .copied()
        .map(|register| {
            let temporary = Temporary::fresh("save");
            tiler.tile_binary(asm::Binary::Mov, temporary, register);
            (temporary, register)
        })
        .collect::<Vec<_>>();

    tiler.inject_multiple_return_argument();

    function
        .statements
        .iter()
        .for_each(|statement| tiler.tile_statement(statement));

    for (temporary, register) in callee_saved {
        tiler.tile_binary(asm::Binary::Mov, register, temporary);
    }

    asm::Function {
        name: function.name,
        statements: tiler.instructions,
        arguments: function.arguments,
        returns: function.returns,
    }
}

impl Tiler {
    fn inject_multiple_return_argument(&mut self) {
        if let Some(temporary) = self.caller_returns {
            self.tile_binary(asm::Binary::Mov, temporary, abi::read_argument(0));
        }
    }

    fn tile_statement(&mut self, statement: &lir::Statement<lir::Fallthrough>) {
        match statement {
            lir::Statement::Label(label) => self.push(Assembly::Label(*label)),
            lir::Statement::Return(returns) => {
                for (index, r#return) in returns.iter().enumerate() {
                    self.tile_binary(
                        asm::Binary::Mov,
                        abi::write_return(self.caller_returns, index),
                        r#return,
                    );
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
                self.tile_binary(asm::Binary::Cmp, left, right);
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

                self.tile_binary(binary, destination, source);
            }
            lir::Statement::Call(function, _, _) if function.is_label(abi::XI_OUT_OF_BOUNDS) => {
                let function = self.tile_expression(function);
                self.push(asm::Assembly::Unary(
                    asm::Unary::Call {
                        arguments: 0,
                        returns: 0,
                    },
                    function,
                ));
            }
            lir::Statement::Call(function, arguments, returns) => {
                #[allow(clippy::needless_collect)]
                let caller_saved = abi::CALLER_SAVED
                    .iter()
                    .copied()
                    .map(|register| {
                        let temporary = Temporary::fresh("save");
                        self.tile_binary(asm::Binary::Mov, temporary, register);
                        (temporary, register)
                    })
                    .collect::<Vec<_>>();

                let offset = if *returns > 2 {
                    self.tile_binary(
                        asm::Binary::Lea,
                        abi::write_argument(0),
                        abi::read_return(self.callee_arguments, 0),
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
                self.push(asm::Assembly::Unary(
                    asm::Unary::Call {
                        arguments: arguments.len(),
                        returns: *returns,
                    },
                    function,
                ));

                for (temporary, register) in caller_saved {
                    self.tile_binary(asm::Binary::Mov, register, temporary)
                }
            }
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
            return self.tile_unary(asm::Unary::Neg, destination, Mutate::No);
        }

        match binary {
            ir::Binary::Add
            | ir::Binary::Sub
            | ir::Binary::And
            | ir::Binary::Or
            | ir::Binary::Xor => {
                let fresh = Temporary::fresh("tile");
                self.tile_binary(asm::Binary::Mov, fresh, destination);
                self.tile_binary(asm::Binary::from(*binary), fresh, source);
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

                let fresh = Temporary::fresh("tile");

                self.tile_binary(asm::Binary::Mov, Register::Rax, destination);
                if cqo {
                    self.push(Assembly::Nullary(asm::Nullary::Cqo));
                }
                self.tile_unary(unary, source, Mutate::Yes);
                self.tile_binary(asm::Binary::Mov, fresh, register);

                operand::Unary::R(fresh)
            }
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
        };

        self.push(Assembly::Binary(binary, operands));
    }

    /// Assumes `unary` operates only on register and memory operands. Immediates will be shuttled.
    fn tile_unary(
        &mut self,
        unary: asm::Unary,
        destination: &lir::Expression,
        mutate: Mutate,
    ) -> operand::Unary<Temporary> {
        let destination = match (self.tile_expression(destination), mutate) {
            (destination @ operand::Unary::I(_), _) => operand::Unary::R(self.shuttle(destination)),
            (destination @ operand::Unary::M(_), Mutate::Yes) => destination,
            (destination @ operand::Unary::R(_), Mutate::Yes) => destination,
            (destination @ operand::Unary::M(_), Mutate::No) => {
                operand::Unary::R(self.shuttle(destination))
            }
            (destination @ operand::Unary::R(_), Mutate::No) => {
                let fresh = Temporary::fresh("tile");
                self.tile_binary(asm::Binary::Mov, fresh, destination);
                operand::Unary::R(fresh)
            }
        };

        self.push(Assembly::Unary(unary, destination));
        destination
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
