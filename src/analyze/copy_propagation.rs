use crate::abi;
use crate::analyze::Analysis;
use crate::data::asm;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::util;
use crate::Map;

#[derive(Default)]
pub struct CopyPropagation;

impl<T: lir::Target> Analysis<lir::Function<T>> for CopyPropagation {
    const BACKWARD: bool = false;

    type Data = Map<Temporary, Temporary>;

    fn default(&self) -> Self::Data {
        Map::default()
    }

    fn transfer(
        &self,
        statement: &<lir::Function<T> as crate::cfg::Function>::Statement,
        output: &mut Self::Data,
    ) {
        match statement {
            lir::Statement::Jump(_)
            | lir::Statement::CJump { .. }
            | lir::Statement::Label(_)
            | lir::Statement::Return(_) => (),
            lir::Statement::Call(_, _, returns) => {
                for r#return in returns {
                    remove(output, r#return);
                }
            }
            lir::Statement::Move {
                destination: lir::Expression::Temporary(destination),
                source: lir::Expression::Temporary(source),
            } => {
                remove(output, destination);
                output.insert(*destination, *source);
            }
            lir::Statement::Move {
                destination: lir::Expression::Temporary(destination),
                source: _,
            } => {
                remove(output, destination);
            }
            lir::Statement::Move { .. } => (),
        }
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        <Self as Analysis<asm::Function<Temporary>>>::merge(self, outputs, input)
    }
}

impl Analysis<asm::Function<Temporary>> for CopyPropagation {
    const BACKWARD: bool = false;

    type Data = Map<Temporary, Temporary>;

    fn default(&self) -> Self::Data {
        Map::default()
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
            asm::Statement::Binary(Lea | Add | Sub | Shl | Mul | And | Or | Xor, operands) => {
                if let util::Or::L(temporary) = operands.destination() {
                    remove(output, &temporary);
                }
            }
            asm::Statement::Binary(Cmp, _) => (),
            asm::Statement::Unary(Hul | Div | Mod, _) | asm::Statement::Nullary(Cqo) => {
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
            asm::Statement::Unary(Push | Pop, operand) => {
                assert_eq!(
                    *operand,
                    operand::Unary::R(Temporary::Register(Register::Rbp))
                );
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
fn remove(output: &mut Map<Temporary, Temporary>, kill: &Temporary) {
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
