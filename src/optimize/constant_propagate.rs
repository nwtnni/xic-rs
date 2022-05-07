use crate::analyze::analyze;
use crate::analyze::Analysis as _;
use crate::analyze::ConstantPropagation;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::asm::Statement;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Temporary;

pub fn propagate(cfg: &mut Cfg<asm::Function<Temporary>>) {
    let mut solution = analyze::<ConstantPropagation, _>(cfg);

    for (label, statements) in cfg.blocks_mut() {
        let mut output = solution.inputs.remove(label).unwrap();

        for statement in statements {
            use asm::Binary::*;
            use asm::Nullary::*;
            use asm::Unary::*;

            let save = statement.clone();

            match statement {
                Statement::Binary(
                    binary @ (Cmp | Mov | Lea | Add | Sub | Shl | And | Or | Xor),
                    operands,
                ) => {
                    let replace = match operands {
                        operand::Binary::RI { .. }
                        | operand::Binary::MI { .. }
                        | operand::Binary::RM { .. } => None,
                        operand::Binary::MR {
                            destination,
                            source,
                        } => output.get(source).and_then(|immediate| {
                            if let Immediate::Integer(integer) = immediate {
                                if i32::try_from(*integer).is_err() && *binary != Mov {
                                    return None;
                                }
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
                            if let Immediate::Integer(integer) = immediate {
                                if i32::try_from(*integer).is_err() && *binary != Mov {
                                    return None;
                                }
                            }

                            Some(operand::Binary::RI {
                                destination: *destination,
                                source: *immediate,
                            })
                        }),
                    };

                    if let Some(replace) = replace {
                        *operands = replace;
                    }
                }
                Statement::Unary(Neg | Mul | Hul | Div | Mod | Call { .. }, _)
                | Statement::Nullary(Nop | Cqo | Ret(_))
                | Statement::Label(_)
                | Statement::Jmp(_)
                | Statement::Jcc(_, _) => (),
            };

            solution.analysis.transfer(&save, &mut output);
        }
    }
}
