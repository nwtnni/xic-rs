use core::fmt;
use std::marker::PhantomData;

use crate::abi;
use crate::analyze::Analysis;
use crate::cfg;
use crate::data::asm;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::util::Or;
use crate::Set;

/// This module technically implements a stronger version of live variable analysis,
/// which seems to be referred to as truly or strongly live variable analysis (or its
/// complement, faint variable analysis):
///
/// - https://pages.cs.wisc.edu/~cs701-1/LectureNotes/trunk/DataflowAnalysisProblems.pdf
/// - https://suif.stanford.edu/~courses/cs243/hws/hw2.pdf
/// - https://www.cl.cam.ac.uk/teaching/1011/L111/general-frameworks.pdf
/// - https://www.rw.cdl.uni-saarland.de/people/joba/private/PA/Slides/backward_4.pdf
///
/// We implement the live analysis instead of the faint analysis for performance:
/// this way, we don't need to eagerly construct and clone the set of all variables
/// in the program.
///
/// Note: we could also switch to (compressed) bitmaps if profiling indicates a bottleneck.
pub struct LiveVariables<T>(PhantomData<T>);

impl<T: Function> Analysis<T> for LiveVariables<T>
where
    T::Statement: fmt::Display,
{
    const BACKWARD: bool = true;

    type Data = Set<Temporary>;

    fn new() -> Self {
        LiveVariables(PhantomData)
    }

    fn default(&self) -> Self::Data {
        Set::default()
    }

    fn transfer(&self, statement: &T::Statement, output: &mut Self::Data) {
        T::transfer(statement, output);
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        input.clear();
        for output in outputs.flatten() {
            input.extend(output);
        }
    }
}

trait Function: cfg::Function {
    fn transfer(statement: &Self::Statement, output: &mut Set<Temporary>);
}

impl Function for asm::Function<Temporary> {
    fn transfer(statement: &Self::Statement, output: &mut Set<Temporary>) {
        match statement {
            asm::Statement::Label(_) | asm::Statement::Jmp(_) | asm::Statement::Jcc(_, _) => {}
            asm::Statement::Nullary(asm::Nullary::Nop) => {}

            asm::Statement::Nullary(asm::Nullary::Cqo) if dead_assembly(Register::Rdx, output) => {
                assert!(dead_assembly(Register::Rax, output));
            }
            asm::Statement::Nullary(asm::Nullary::Cqo) => {
                output.remove(&Temporary::Register(Register::Rdx));

                // Both uses and defines `rax`
                output.insert(Temporary::Register(Register::Rax));
            }

            asm::Statement::Nullary(asm::Nullary::Ret(returns)) => {
                for r#return in abi::RETURN.iter().take(*returns) {
                    output.insert(Temporary::Register(*r#return));
                }

                // `rsp` must be live throughout the entire program
                output.insert(Temporary::Register(Register::rsp()));
            }

            asm::Statement::Binary(
                asm::Binary::Mov
                | asm::Binary::Lea
                | asm::Binary::Add
                | asm::Binary::Sub
                | asm::Binary::Shl
                | asm::Binary::Mul
                | asm::Binary::And
                | asm::Binary::Or
                | asm::Binary::Xor,
                operands,
            ) if dead_assembly(operands.destination(), output) => (),
            asm::Statement::Binary(binary, operands) => {
                match (binary, operands.destination()) {
                    (asm::Binary::Mov | asm::Binary::Lea, Or::L(temporary)) => {
                        output.remove(&temporary);
                    }
                    (
                        asm::Binary::Cmp
                        | asm::Binary::Add
                        | asm::Binary::Sub
                        | asm::Binary::Shl
                        | asm::Binary::Mul
                        | asm::Binary::And
                        | asm::Binary::Or
                        | asm::Binary::Xor,
                        Or::L(temporary),
                    ) => {
                        // Both uses and defines `temporary`
                        output.insert(temporary);
                    }
                    (_, Or::R(memory)) => {
                        memory.map(|temporary| output.insert(*temporary));
                    }
                }

                operands.source().map(|temporary| output.insert(*temporary));
            }

            // Special case: `_xi_out_of_bounds` diverges, so nothing after is reachable.
            asm::Statement::Unary(
                asm::Unary::Call { .. },
                operand::Unary::I(Immediate::Label(Label::Fixed(label))),
            ) if symbol::resolve(*label) == abi::XI_OUT_OF_BOUNDS => {
                output.clear();
            }
            asm::Statement::Unary(asm::Unary::Call { arguments, returns }, operand) => {
                for r#return in abi::RETURN.iter().take(*returns) {
                    output.remove(&Temporary::Register(*r#return));
                }

                for argument in abi::ARGUMENT.iter().take(*arguments) {
                    output.insert(Temporary::Register(*argument));
                }

                operand.map(|temporary| output.insert(*temporary));
            }

            asm::Statement::Unary(asm::Unary::Neg, operand) if dead_assembly(*operand, output) => {}
            asm::Statement::Unary(asm::Unary::Neg, operand) => {
                // Both uses and defines `operand`
                operand.map(|temporary| output.insert(*temporary));
            }

            // We don't check `div` and `mod` as they can have side effects (x / 0, x % 0)
            asm::Statement::Unary(asm::Unary::Hul, _) if dead_assembly(Register::Rdx, output) => (),
            asm::Statement::Unary(
                unary @ (asm::Unary::Hul | asm::Unary::Div | asm::Unary::Mod),
                operand,
            ) => {
                if matches!(unary, asm::Unary::Hul | asm::Unary::Mod) {
                    output.remove(&Temporary::Register(Register::Rdx));
                }

                // Both uses and defines `rax`
                output.insert(Temporary::Register(Register::Rax));

                if matches!(unary, asm::Unary::Div | asm::Unary::Mod) {
                    output.insert(Temporary::Register(Register::Rdx));
                }

                operand.map(|temporary| output.insert(*temporary));
            }
            asm::Statement::Unary(asm::Unary::Push | asm::Unary::Pop, operand) => {
                assert_eq!(
                    *operand,
                    operand::Unary::R(Temporary::Register(Register::Rbp))
                );
            }
        }
    }
}

fn dead_assembly<I: Into<operand::Unary<Temporary>>>(
    destination: I,
    live: &Set<Temporary>,
) -> bool {
    match destination.into() {
        // Special case: another option is to mark all callee-saved registers live at
        // the `ret` instruction, just like we do for `rsp`. However, because the linear
        // scan register allocator operates on coarse live ranges and not fine intervals,
        // each register ends up with a live range spanning the entire program.
        //
        // So instead, we pretend callee-saved registers are dead at the end, and preserve
        // otherwise dead assignments of the form `mov callee_saved_register, save` here.
        operand::Unary::R(Temporary::Register(register)) if register.is_callee_saved() => false,

        // Without alias analysis, we can't say anything about memory addresses.
        // Conservatively mark all memory accesses as live (and observable).
        operand::Unary::M(_) => false,
        operand::Unary::R(temporary) => !live.contains(&temporary),
        operand::Unary::I(_) => unreachable!(),
    }
}

impl<T: lir::Target> Function for lir::Function<T> {
    fn transfer(statement: &Self::Statement, output: &mut Set<Temporary>) {
        match statement {
            lir::Statement::Jump(_) | lir::Statement::Label(_) => (),
            lir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                transfer_expression(left, output);
                transfer_expression(right, output);
            }
            lir::Statement::Call(function, arguments, returns) => {
                for r#return in returns {
                    output.remove(r#return);
                }

                transfer_expression(function, output);
                for argument in arguments {
                    transfer_expression(argument, output);
                }
            }
            lir::Statement::Move {
                destination,
                source: _,
            } if dead_lir(destination, output) => (),
            lir::Statement::Move {
                destination,
                source,
            } => {
                match destination {
                    lir::Expression::Immediate(_) | lir::Expression::Binary(_, _, _) => {
                        unreachable!()
                    }
                    lir::Expression::Memory(address) => transfer_expression(address, output),
                    lir::Expression::Temporary(temporary) => {
                        output.remove(temporary);
                    }
                }
                transfer_expression(source, output);
            }
            lir::Statement::Return(returns) => {
                for r#return in returns {
                    transfer_expression(r#return, output);
                }
            }
        }
    }
}

fn transfer_expression(expression: &lir::Expression, output: &mut Set<Temporary>) {
    match expression {
        lir::Expression::Immediate(_) => (),
        lir::Expression::Temporary(temporary) => {
            output.insert(*temporary);
        }
        lir::Expression::Memory(address) => transfer_expression(address, output),
        lir::Expression::Binary(_, left, right) => {
            transfer_expression(left, output);
            transfer_expression(right, output);
        }
    }
}

fn dead_lir(expression: &lir::Expression, live: &Set<Temporary>) -> bool {
    match expression {
        lir::Expression::Immediate(_) | lir::Expression::Binary(_, _, _) => unreachable!(),
        lir::Expression::Temporary(temporary) => !live.contains(temporary),
        lir::Expression::Memory(_) => false,
    }
}
