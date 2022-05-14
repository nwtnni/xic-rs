use crate::abi;
use crate::analyze::Analysis;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::util;
use crate::Map;

pub struct ConstantPropagation;

impl Analysis<asm::Function<Temporary>> for ConstantPropagation {
    const BACKWARD: bool = false;

    type Data = Map<Temporary, Immediate>;

    fn new() -> Self {
        ConstantPropagation
    }

    fn default(&self) -> Self::Data {
        Map::default()
    }

    fn transfer(&self, statement: &asm::Statement<Temporary>, output: &mut Self::Data) {
        use asm::Binary::*;
        use asm::Nullary::*;
        use asm::Unary::*;

        match statement {
            asm::Statement::Binary(Mov, operands) => match operands {
                operand::Binary::RI {
                    destination,
                    source,
                } => {
                    output.insert(*destination, *source);
                }
                operand::Binary::RR {
                    destination,
                    source,
                } => {
                    if let Some(immediate) = output.get(source).copied() {
                        output.insert(*destination, immediate);
                    } else {
                        output.remove(destination);
                    }
                }
                operand::Binary::RM { destination, .. } => {
                    output.remove(destination);
                }
                operand::Binary::MR { .. } | operand::Binary::MI { .. } => (),
            },
            asm::Statement::Binary(Lea, operands) => {
                if let util::Or::L(temporary) = operands.destination() {
                    output.remove(&temporary);
                }
            }
            asm::Statement::Binary(binary @ (Add | Sub | Shl | Mul | And | Or | Xor), operands) => {
                match operands {
                    operand::Binary::RI {
                        destination,
                        source,
                    } => {
                        if let Some(immediate) = output
                            .get(destination)
                            .and_then(|immediate| fold_binary(binary, immediate, source))
                        {
                            output.insert(*destination, immediate);
                        } else {
                            output.remove(destination);
                        }
                    }
                    operand::Binary::RM { destination, .. } => {
                        output.remove(destination);
                    }
                    operand::Binary::RR {
                        destination,
                        source,
                    } => {
                        if let Some(immediate) =
                            output.get(destination).zip(output.get(source)).and_then(
                                |(destination, source)| fold_binary(binary, destination, source),
                            )
                        {
                            output.insert(*destination, immediate);
                        } else {
                            output.remove(destination);
                        }
                    }
                    operand::Binary::MI { .. } | operand::Binary::MR { .. } => (),
                }
            }
            asm::Statement::Binary(Cmp, _) => (),
            asm::Statement::Unary(unary @ (Hul | Div | Mod), operand) => {
                let immediate = match operand {
                    operand::Unary::I(immediate) => Some(immediate),
                    operand::Unary::R(temporary) => output.get(temporary),
                    operand::Unary::M(_) => None,
                };

                if let Some((rax, rdx)) = output
                    .get(&Temporary::Register(Register::Rax))
                    .zip(immediate)
                    .and_then(|(destination, source)| fold_unary(unary, destination, source))
                {
                    output.insert(Temporary::Register(Register::Rax), rax);
                    output.insert(Temporary::Register(Register::Rdx), rdx);
                } else {
                    output.remove(&Temporary::Register(Register::Rax));
                    output.remove(&Temporary::Register(Register::Rdx));
                }
            }
            asm::Statement::Unary(Neg, operand::Unary::R(temporary)) => {
                if let Some(Immediate::Integer(integer)) = output.get(temporary).copied() {
                    output.insert(*temporary, Immediate::Integer(-integer));
                } else {
                    output.remove(temporary);
                }
            }
            asm::Statement::Unary(Neg, operand::Unary::I(_) | operand::Unary::M(_)) => (),
            asm::Statement::Unary(
                Call {
                    arguments: _,
                    returns,
                },
                _,
            ) => {
                for register in abi::CALLER_SAVED
                    .iter()
                    .chain(abi::RETURN.iter().take(*returns))
                {
                    output.remove(&Temporary::Register(*register));
                }
            }

            asm::Statement::Nullary(Cqo) => {
                output.insert(Temporary::Register(Register::Rdx), Immediate::Integer(0));
            }
            asm::Statement::Nullary(Nop | Ret(_))
            | asm::Statement::Label(_)
            | asm::Statement::Jmp(_)
            | asm::Statement::Jcc(_, _) => (),
        }
    }

    fn merge<'a, I>(&self, mut outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        input.clear();
        input.extend(outputs.next().into_iter().flatten().flatten());

        for output in outputs {
            let output = match output {
                Some(output) => output,
                None => {
                    input.clear();
                    return;
                }
            };

            input.retain(|temporary, old| match output.get(temporary) {
                Some(new) if old == new => true,
                Some(_) | None => false,
            });
        }
    }
}

fn fold_binary(
    binary: &asm::Binary,
    destination: &Immediate,
    source: &Immediate,
) -> Option<Immediate> {
    let (destination, source) = match (destination, source) {
        (Immediate::Integer(destination), Immediate::Integer(source)) => (*destination, *source),
        (_, _) => return None,
    };

    let integer = match binary {
        asm::Binary::Add => destination.wrapping_add(source),
        asm::Binary::Sub => destination.wrapping_sub(source),
        asm::Binary::Shl => destination << source,
        asm::Binary::Mul => destination.wrapping_mul(source),
        asm::Binary::And => destination & source,
        asm::Binary::Or => destination | source,
        asm::Binary::Xor => destination ^ source,
        asm::Binary::Cmp | asm::Binary::Mov | asm::Binary::Lea => return None,
    };

    Some(Immediate::Integer(integer))
}

fn fold_unary(
    unary: &asm::Unary,
    destination: &Immediate,
    source: &Immediate,
) -> Option<(Immediate, Immediate)> {
    let (destination, source) = match (destination, source) {
        (Immediate::Integer(destination), Immediate::Integer(source)) => (*destination, *source),
        (_, _) => return None,
    };

    let (rax, rdx) = match unary {
        asm::Unary::Neg | asm::Unary::Call { .. } => return None,
        asm::Unary::Hul => (
            destination.wrapping_mul(source),
            (((destination as i128) * (source as i128)) >> 64) as i64,
        ),
        asm::Unary::Div | asm::Unary::Mod if source == 0 => return None,
        asm::Unary::Div | asm::Unary::Mod => (destination / source, destination % source),
    };

    Some((Immediate::Integer(rax), Immediate::Integer(rdx)))
}
