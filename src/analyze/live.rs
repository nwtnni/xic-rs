use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::abi;
use crate::analyze::Analysis;
use crate::analyze::Backward;
use crate::cfg;
use crate::data::asm;
use crate::data::asm::Assembly;
use crate::data::operand;
use crate::data::operand::Register;
use crate::data::operand::Temporary;

pub struct LiveVariable<T>(PhantomData<T>);

impl<T> Default for LiveVariable<T> {
    fn default() -> Self {
        LiveVariable(PhantomData)
    }
}

impl<T: Function> Analysis<T> for LiveVariable<T> {
    type Data = BTreeSet<Temporary>;
    type Direction = Backward;

    fn default(&mut self, _: &crate::cfg::Cfg<T>, _: &crate::data::operand::Label) -> Self::Data {
        BTreeSet::new()
    }

    fn merge(&mut self, output: &Self::Data, input: &mut Self::Data) {
        input.extend(output);
    }

    fn transfer(
        &mut self,
        statements: &[T::Statement],
        input: &Self::Data,
        output: &mut Self::Data,
    ) -> bool {
        output.extend(input);
        let before = output.len();

        for statement in statements {
            T::transfer(statement, output);
        }

        let after = output.len();
        before != after
    }
}

trait Function: cfg::Function {
    fn transfer(statement: &Self::Statement, output: &mut BTreeSet<Temporary>);
}

impl Function for asm::Function<Temporary> {
    fn transfer(statement: &Self::Statement, output: &mut BTreeSet<Temporary>) {
        match statement {
            Assembly::Label(_) | Assembly::Jmp(_) | Assembly::Jcc(_, _) => {}
            Assembly::Nullary(asm::Nullary::Cqo) => {
                output.remove(&Temporary::Register(Register::Rdx));

                // Both uses and defines `rax`:
                // output.remove(&Temporary::Register(Register::Rax));
                // output.insert(Temporary::Register(Register::Rax));
            }
            Assembly::Nullary(asm::Nullary::Ret(returns, caller_returns)) => {
                for r#return in 0..*returns {
                    match abi::write_return(*caller_returns, r#return) {
                        operand::Unary::I(_) => (),
                        operand::Unary::R(temporary) => {
                            output.insert(temporary);
                        }
                        operand::Unary::M(memory) => {
                            memory.map(|temporary| output.insert(*temporary));
                        }
                    }
                }
            }
            Assembly::Binary(binary, operands) => {
                use asm::Binary::*;

                match (binary, operands.destination()) {
                    (_, operand::Unary::I(_)) => (),
                    (Cmp, operand::Unary::R(_)) => (),
                    (Add | Sub | And | Or | Xor | Mov | Lea, operand::Unary::R(temporary)) => {
                        output.remove(&temporary);
                    }
                    (_, operand::Unary::M(memory)) => {
                        memory.map(|temporary| output.insert(*temporary));
                    }
                }

                match operands.source() {
                    operand::Unary::I(_) => (),
                    operand::Unary::R(temporary) => {
                        output.insert(temporary);
                    }
                    operand::Unary::M(memory) => {
                        memory.map(|temporary| output.insert(*temporary));
                    }
                }
            }
            Assembly::Unary(
                asm::Unary::Call {
                    arguments,
                    returns: _,
                },
                operand,
            ) => {
                // Return registers `rax` and `rdx` are also caller saved and therefore defined.
                for register in abi::CALLER_SAVED {
                    output.remove(&Temporary::Register(*register));
                }

                for argument in 0..*arguments {
                    match abi::write_argument(argument) {
                        operand::Unary::I(_) => (),
                        operand::Unary::R(temporary) => {
                            output.insert(temporary);
                        }
                        operand::Unary::M(memory) => {
                            memory.map(|temporary| output.insert(*temporary));
                        }
                    }
                }

                match operand {
                    operand::Unary::I(_) => (),
                    operand::Unary::R(temporary) => {
                        output.insert(*temporary);
                    }
                    operand::Unary::M(memory) => {
                        memory.map(|temporary| output.insert(*temporary));
                    }
                }
            }
            Assembly::Unary(asm::Unary::Neg, operand) => match operand {
                operand::Unary::I(_) => (),
                operand::Unary::R(_) => {
                    // Both uses and defines the temporary:
                    // output.remove(&temporary);
                    // output.insert(temporary);
                }
                operand::Unary::M(memory) => {
                    memory.map(|temporary| output.insert(*temporary));
                }
            },
            Assembly::Unary(asm::Unary::Mul | asm::Unary::Div(_), operand) => {
                output.remove(&Temporary::Register(Register::Rdx));

                // Both uses and defines `rax`:
                // output.remove(&Temporary::Register(Register::Rax));
                // output.insert(Temporary::Register(Register::Rax));

                match operand {
                    operand::Unary::I(_) => (),
                    operand::Unary::R(temporary) => {
                        output.insert(*temporary);
                    }
                    operand::Unary::M(memory) => {
                        memory.map(|temporary| output.insert(*temporary));
                    }
                }
            }
        }
    }
}
