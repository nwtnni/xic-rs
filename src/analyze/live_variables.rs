use core::fmt;
use std::cmp;
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
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::symbol;

pub struct LiveVariables<T>(PhantomData<T>);

impl<T: Function> Analysis<T> for LiveVariables<T>
where
    T::Statement: fmt::Display,
{
    type Data = BTreeSet<Temporary>;
    type Direction = Backward;

    fn new(_: &Cfg<T>) -> Self {
        LiveVariables(PhantomData)
    }

    fn default(&self, _: &Cfg<T>, _: &Label) -> Self::Data {
        BTreeSet::new()
    }

    fn merge(&self, output: &Self::Data, input: &mut Self::Data) {
        input.extend(output);
    }

    fn transfer(&self, statement: &T::Statement, output: &mut Self::Data) {
        T::transfer(statement, output);
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
                output.insert(Temporary::Register(Register::Rax));
            }
            Assembly::Nullary(asm::Nullary::Ret(returns)) => {
                // ABI-specific value (2)
                for r#return in 0..cmp::min(2, *returns) {
                    match abi::write_return(None, r#return) {
                        operand::Unary::I(_) => (),
                        operand::Unary::M(_) => unreachable!(),
                        operand::Unary::R(temporary) => {
                            output.insert(temporary);
                        }
                    }
                }
                output.insert(Temporary::Register(Register::rsp()));
            }
            Assembly::Binary(binary, operands) => {
                use asm::Binary::*;

                match (binary, operands.destination()) {
                    (_, operand::Unary::I(_)) => (),
                    (Mov | Lea, operand::Unary::R(temporary)) => {
                        output.remove(&temporary);
                    }
                    (Cmp | Add | Sub | And | Or | Xor, operand::Unary::R(temporary)) => {
                        // Both uses and defines `temporary`
                        // output.remove(&temporary);
                        output.insert(temporary);
                    }
                    (_, operand::Unary::M(memory)) => {
                        memory.map(|temporary| output.insert(*temporary));
                    }
                }

                operands.source().map(|temporary| output.insert(*temporary));
            }
            // Special case: `_xi_out_of_bounds` diverges, so nothing after is reachable.
            Assembly::Unary(
                asm::Unary::Call { .. },
                operand::Unary::I(Immediate::Label(Label::Fixed(label))),
            ) if symbol::resolve(*label) == abi::XI_OUT_OF_BOUNDS => {
                output.clear();
            }
            Assembly::Unary(asm::Unary::Call { arguments, returns }, operand) => {
                for r#return in 0..*returns {
                    match abi::read_return(*arguments, r#return) {
                        operand::Unary::I(_) => (),
                        operand::Unary::M(_) => (),
                        operand::Unary::R(temporary) => {
                            output.remove(&temporary);
                        }
                    }
                }

                output.insert(Temporary::Register(Register::rsp()));

                for argument in 0..*arguments {
                    abi::write_argument(argument).map(|temporary| output.insert(*temporary));
                }

                operand.map(|temporary| output.insert(*temporary));
            }
            Assembly::Unary(asm::Unary::Neg, operand) => {
                operand.map(|temporary| output.insert(*temporary));
            }
            Assembly::Unary(
                unary @ (asm::Unary::Mul | asm::Unary::Hul | asm::Unary::Div | asm::Unary::Mod),
                operand,
            ) => {
                output.remove(&Temporary::Register(Register::Rdx));

                // Both uses and defines `rax`:
                // output.remove(&Temporary::Register(Register::Rax));
                output.insert(Temporary::Register(Register::Rax));

                if let asm::Unary::Div | asm::Unary::Mod = unary {
                    output.insert(Temporary::Register(Register::Rdx));
                }

                operand.map(|temporary| output.insert(*temporary));
            }
        }
    }
}
