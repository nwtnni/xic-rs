use std::borrow::Cow;

use crate::analyze::analyze;
use crate::analyze::Analysis as _;
use crate::analyze::ConstantPropagation;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::asm::Statement;
use crate::data::operand;
use crate::data::operand::Temporary;
use crate::util;

pub fn propagate_assembly(cfg: &mut Cfg<asm::Function<Temporary>>) {
    log::info!(
        "[{}] Propagating constants in {}...",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name(),
    );
    util::time!(
        "[{}] Done propagating constants in {}",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name(),
    );

    let mut solution = analyze::<ConstantPropagation, _>(cfg);
    let mut propagated = 0;

    for (label, statements) in cfg.blocks_mut() {
        let mut output = solution.inputs.remove(label).unwrap();

        for statement in statements {
            use asm::Binary::*;
            use asm::Nullary::*;
            use asm::Unary::*;

            let statement = match statement {
                Statement::Binary(
                    binary @ (Cmp | Mov | Lea | Add | Sub | Shl | Mul | And | Or | Xor),
                    operands,
                ) => {
                    // TODO: propagate constants to temporaries inside memory operands
                    let propagate = match operands {
                        operand::Binary::RI { .. }
                        | operand::Binary::MI { .. }
                        | operand::Binary::RM { .. } => None,
                        operand::Binary::MR {
                            destination,
                            source,
                        } => output.get(source).and_then(|immediate| {
                            if immediate.is_64_bit() {
                                return None;
                            }

                            Some(operand::Binary::MI {
                                destination: *destination,
                                source: *immediate,
                            })
                        }),
                        operand::Binary::RR {
                            destination,
                            source,
                        } => output.get(source).and_then(|immediate| {
                            if immediate.is_64_bit() && *binary != Mov {
                                return None;
                            }

                            Some(operand::Binary::RI {
                                destination: *destination,
                                source: *immediate,
                            })
                        }),
                    };

                    match propagate {
                        Some(propagate) if *operands != propagate => {
                            let transfer = Statement::Binary(*binary, *operands);
                            propagated += 1;
                            *operands = propagate;
                            log::trace!("Replaced {} with {}", transfer, statement);
                            Cow::Owned(transfer)
                        }
                        None | Some(_) => Cow::Borrowed(statement),
                    }
                }
                Statement::Unary(Neg | Hul | Div | Mod | Call { .. }, _)
                | Statement::Nullary(Nop | Cqo | Ret(_))
                | Statement::Label(_)
                | Statement::Jmp(_)
                | Statement::Jcc(_, _) => Cow::Borrowed(statement),
            };

            solution.analysis.transfer(&statement, &mut output);
        }
    }

    log::debug!("Propagated {} assembly constants!", propagated);
}
