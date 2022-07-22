use std::fmt;

use crate::analyze::analyze_default;
use crate::analyze::Analysis;
use crate::analyze::CopyPropagation;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::asm::Statement;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Temporary;
use crate::util;
use crate::Map;

pub fn propagate_lir<T: lir::Target>(cfg: &mut Cfg<lir::Function<T>>) {
    log::info!(
        "[{}] Propagating copies in {}...",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );
    util::time!(
        "[{}] Done propagating copies in {}",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );

    let mut solution = analyze_default::<CopyPropagation, _>(cfg);
    let mut propagated = 0;

    for (label, statements) in cfg.blocks_mut() {
        let mut output = solution.inputs.remove(label).unwrap();

        for statement in statements {
            let save = statement.clone();

            let mut traverse = |temporary| traverse(&output, &save, &mut propagated, temporary);

            match statement {
                lir::Statement::Jump(_) | lir::Statement::Label(_) => (),
                lir::Statement::CJump {
                    condition: _,
                    left,
                    right,
                    r#true: _,
                    r#false: _,
                } => {
                    propagate_lir_expression(left, &mut traverse);
                    propagate_lir_expression(right, &mut traverse);
                }
                lir::Statement::Call(function, arguments, _) => {
                    propagate_lir_expression(function, &mut traverse);
                    for argument in arguments {
                        propagate_lir_expression(argument, &mut traverse);
                    }
                }
                lir::Statement::Move {
                    destination: lir::Expression::Temporary(_),
                    source,
                } => {
                    propagate_lir_expression(source, &mut traverse);
                }
                lir::Statement::Move {
                    destination: lir::Expression::Memory(address),
                    source,
                } => {
                    propagate_lir_expression(address, &mut traverse);
                    propagate_lir_expression(source, &mut traverse);
                }
                lir::Statement::Move { .. } => unreachable!(),
                lir::Statement::Return(returns) => {
                    for r#return in returns {
                        propagate_lir_expression(r#return, &mut traverse);
                    }
                }
            }

            <CopyPropagation as Analysis<lir::Function<T>>>::transfer(
                &solution.analysis,
                &save,
                &mut output,
            );
        }
    }

    log::debug!("Propagated {} copies!", propagated);
}

fn propagate_lir_expression<F: FnMut(Temporary) -> Temporary>(
    expression: &mut lir::Expression,
    apply: &mut F,
) {
    match expression {
        lir::Expression::Immediate(_) => (),
        lir::Expression::Temporary(temporary) => *temporary = apply(*temporary),
        lir::Expression::Memory(address) => propagate_lir_expression(address, apply),
        lir::Expression::Binary(_, left, right) => {
            propagate_lir_expression(left, apply);
            propagate_lir_expression(right, apply);
        }
    }
}

pub fn propagate_assembly(cfg: &mut Cfg<asm::Function<Temporary>>) {
    log::info!(
        "[{}] Propagating copies in {}...",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );
    util::time!(
        "[{}] Done propagating copies in {}",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );

    let mut solution = analyze_default::<CopyPropagation, _>(cfg);
    let mut propagated = 0;

    for (label, statements) in cfg.blocks_mut() {
        let mut output = solution.inputs.remove(label).unwrap();

        for statement in statements {
            use asm::Binary::*;
            use asm::Nullary::*;
            use asm::Unary::*;

            let save = statement.clone();

            let mut traverse = |temporary| traverse(&output, &save, &mut propagated, temporary);

            match statement {
                // `cmp` is a bit special because it doesn't modify its destination.
                // So it should be fine to rewrite something like this:
                //
                // ```
                // mov t0, a     mov t0, a
                // mov t1, b  -> mov t1, b
                // cmp t0, t1    cmp a, b
                // ```
                asm::Statement::Binary(
                    Cmp,
                    operand::Binary::RR {
                        destination,
                        source,
                    },
                ) => {
                    *destination = traverse(*destination);
                    *source = traverse(*source);
                }
                asm::Statement::Binary(
                    Cmp,
                    operand::Binary::RM {
                        destination,
                        source,
                    },
                ) => {
                    *destination = traverse(*destination);
                    *source = source.map(|temporary| traverse(*temporary));
                }
                asm::Statement::Binary(Cmp, operand::Binary::RI { destination, .. }) => {
                    *destination = traverse(*destination);
                }

                Statement::Binary(
                    Cmp | Mov | Lea | Add | Sub | Shl | Mul | And | Or | Xor,
                    operands,
                ) => {
                    let memory = match operands {
                        operand::Binary::RI { .. } => None,
                        operand::Binary::MI { destination, .. } => Some(destination),
                        operand::Binary::MR {
                            destination,
                            source,
                        } => {
                            *source = traverse(*source);
                            Some(destination)
                        }
                        operand::Binary::RM { source, .. } => Some(source),
                        operand::Binary::RR { source, .. } => {
                            *source = traverse(*source);
                            None
                        }
                    };

                    if let Some(memory) = memory {
                        *memory = memory.map(|temporary| traverse(*temporary))
                    }
                }
                Statement::Unary(Hul | Div | Mod | Call { .. }, operand::Unary::R(source)) => {
                    *source = traverse(*source);
                }

                Statement::Unary(Neg | Hul | Div | Mod | Call { .. } | Push | Pop, _)
                | Statement::Nullary(Nop | Cqo | Ret(_))
                | Statement::Label(_)
                | Statement::Jmp(_)
                | Statement::Jcc(_, _) => (),
            };

            <CopyPropagation as Analysis<asm::Function<Temporary>>>::transfer(
                &solution.analysis,
                &save,
                &mut output,
            );
        }
    }

    log::debug!("Propagated {} copies!", propagated);
}

fn traverse<T: fmt::Display>(
    output: &Map<Temporary, Temporary>,
    statement: T,
    propagated: &mut usize,
    temporary: Temporary,
) -> Temporary {
    let mut previous = temporary;
    loop {
        match output.get(&previous) {
            Some(next) => previous = *next,
            None if previous == temporary => return previous,
            None => {
                log::trace!(
                    "Replaced {} with {} in statement: {}",
                    temporary,
                    previous,
                    statement,
                );
                *propagated += 1;
                return previous;
            }
        }
    }
}
