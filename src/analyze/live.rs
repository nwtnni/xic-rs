use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::abi;
use crate::analyze::Analysis;
use crate::analyze::Backward;
use crate::cfg;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::asm::Assembly;
use crate::data::operand;
use crate::data::operand::Label;
use crate::data::operand::Register;
use crate::data::operand::Temporary;

pub struct LiveVariable<T>(PhantomData<T>);

impl<T: Function> Analysis<T> for LiveVariable<T> {
    type Data = BTreeSet<Temporary>;
    type Direction = Backward;

    fn new(_: &Cfg<T>) -> Self {
        LiveVariable(PhantomData)
    }

    fn default(&self, _: &Cfg<T>, _: &Label) -> Self::Data {
        BTreeSet::new()
    }

    fn merge(&self, output: &Self::Data, input: &mut Self::Data) {
        input.extend(output);
    }

    fn transfer(&self, statement: &T::Statement, output: &mut Self::Data) -> bool {
        let before = output.len();
        T::transfer(statement, output);
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
            Assembly::Unary(asm::Unary::Call { arguments, returns }, operand) => {
                for r#return in 0..*returns {
                    match abi::read_return(*arguments, r#return) {
                        operand::Unary::I(_) => (),
                        operand::Unary::M(_) => (),
                        operand::Unary::R(temporary) => {
                            // Note: subset of caller saved, will be inserted below
                            output.remove(&temporary);
                        }
                    }
                }

                for register in abi::CALLER_SAVED {
                    output.insert(Temporary::Register(*register));
                }

                for argument in 0..*arguments {
                    match abi::write_argument(argument) {
                        operand::Unary::I(_) => (),
                        operand::Unary::R(temporary) => {
                            // Note: also subset of caller saved, inserted above
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
            Assembly::Unary(
                unary @ (asm::Unary::Mul | asm::Unary::Hul | asm::Unary::Div | asm::Unary::Mod),
                operand,
            ) => {
                output.remove(&Temporary::Register(Register::Rdx));

                // Both uses and defines `rax`:
                // output.remove(&Temporary::Register(Register::Rax));
                // output.insert(Temporary::Register(Register::Rax));

                if let asm::Unary::Div | asm::Unary::Mod = unary {
                    output.insert(Temporary::Register(Register::Rdx));
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
        }
    }
}
