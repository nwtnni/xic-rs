use std::collections::BTreeMap;

use crate::abi;
use crate::analyze::Forward;
use crate::api::analyze::Analysis;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::util;

pub struct CopyPropagation;

impl Analysis<asm::Function<Temporary>> for CopyPropagation {
    type Data = BTreeMap<Temporary, Temporary>;

    type Direction = Forward;

    fn new(_: &crate::cfg::Cfg<asm::Function<Temporary>>) -> Self {
        CopyPropagation
    }

    fn default(&self) -> Self::Data {
        BTreeMap::new()
    }

    fn transfer(&self, statement: &asm::Statement<Temporary>, output: &mut Self::Data) {
        use asm::Binary::*;
        use asm::Nullary::*;
        use asm::Unary::*;

        match statement {
            asm::Statement::Binary(Mov, operands) => match operands {
                operand::Binary::RR {
                    destination,
                    source,
                } => {
                    remove(output, destination);

                    // Special case: reserve callee-saved temporaries for register allocation
                    if !matches!(source, Temporary::Register(register) if register.is_callee_saved())
                    {
                        output.insert(*destination, *source);
                    }
                }
                operand::Binary::RI { destination, .. }
                | operand::Binary::RM { destination, .. } => {
                    remove(output, destination);
                }
                operand::Binary::MR { .. } | operand::Binary::MI { .. } => (),
            },
            asm::Statement::Binary(Lea | Add | Sub | Shl | And | Or | Xor, operands) => {
                if let util::Or::L(temporary) = operands.destination() {
                    remove(output, &temporary);
                }
            }
            asm::Statement::Binary(Cmp, _) => (),
            asm::Statement::Unary(Mul | Hul | Div | Mod, _) | asm::Statement::Nullary(Cqo) => {
                remove(output, &Temporary::Register(Register::Rax));
                remove(output, &Temporary::Register(Register::Rdx));
            }
            asm::Statement::Unary(Neg, operand) => {
                if let operand::Unary::R(temporary) = operand {
                    remove(output, temporary);
                }
            }
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
                    remove(output, &Temporary::Register(*register))
                }
            }
            asm::Statement::Nullary(Nop | Ret(_))
            | asm::Statement::Label(_)
            | asm::Statement::Jmp(_)
            | asm::Statement::Jcc(_, _) => (),
        }
    }

    fn merge<'a, I>(&'a self, mut outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = &'a Self::Data>,
        Self::Data: 'a,
    {
        input.clear();
        input.extend(outputs.next().into_iter().flatten());

        let mut stack = Vec::new();

        for output in outputs {
            stack.extend(
                input
                    .iter()
                    .filter_map(|(temporary, old)| match output.get(temporary) {
                        Some(new) if old == new => None,
                        Some(_) | None => Some(*temporary),
                    }),
            );

            stack
                .drain(..)
                .for_each(|temporary| remove(input, &temporary));
        }
    }
}

/// Remove the entry for `kill`, as well as all entries that recursively
/// depend on it. For example, given the following sequence of statements:
///
/// ```text
/// {}
/// mov a, b
/// {a: b}
/// mov c, a
/// {c: a, a: b}
/// mov d, a
/// {d: a, c: a, a: b}
/// mov b, 1
/// {}
/// ```
fn remove(output: &mut BTreeMap<Temporary, Temporary>, kill: &Temporary) {
    output.remove(kill);

    let mut stack = vec![*kill];

    while let Some(killed) = stack.pop() {
        output.retain(|kill, _killed| {
            if killed != *_killed {
                return true;
            }

            stack.push(*kill);
            false
        });
    }
}
