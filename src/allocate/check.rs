use crate::abi;
use crate::analyze::Analysis;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::Map;

// https://cfallin.org/blog/2021/03/15/cranelift-isel-3/
struct ValidAllocation<const LINEAR: bool> {
    pub allocated: Map<Temporary, Register>,
    pub spilled: Map<Temporary, usize>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Location {
    Stack(usize),
    Register(Register),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Value {
    Conflict,
    Temporary(Temporary),
    Unknown,
}

impl Value {
    fn merge(&self, value: &Self) -> Self {
        match (self, value) {
            (Value::Conflict, _) | (_, Value::Conflict) => Value::Conflict,
            (Value::Unknown, _) | (_, Value::Unknown) => Value::Unknown,
            (Value::Temporary(left), Value::Temporary(right)) if left == right => *self,
            (Value::Temporary(_), Value::Temporary(_)) => Value::Conflict,
        }
    }
}

impl<const LINEAR: bool> Analysis<asm::Function<Temporary>> for ValidAllocation<LINEAR> {
    const BACKWARD: bool = false;

    type Data = Map<Location, Value>;

    fn new() -> Self {
        todo!()
    }

    fn default(&self) -> Self::Data {
        Map::default()
    }

    fn transfer(&self, statement: &asm::Statement<Temporary>, output: &mut Self::Data) {
        match statement {
            asm::Statement::Label(_) | asm::Statement::Jmp(_) | asm::Statement::Jcc(_, _) => (),
            asm::Statement::Binary(binary, operands) => {
                let access = match binary {
                    asm::Binary::Add
                    | asm::Binary::Sub
                    | asm::Binary::Mul
                    | asm::Binary::And
                    | asm::Binary::Or
                    | asm::Binary::Xor
                    | asm::Binary::Shl => Access::ReadWrite,
                    asm::Binary::Cmp => Access::Read,
                    asm::Binary::Mov | asm::Binary::Lea => Access::Write,
                };

                self.transfer_binary(output, access, operands)
            }
            asm::Statement::Unary(unary, operand) => match unary {
                asm::Unary::Push => {
                    self.transfer_unary(output, Access::Write, operand);
                }
                asm::Unary::Pop => {
                    self.transfer_unary(output, Access::Read, operand);
                }
                asm::Unary::Neg => {
                    self.transfer_unary(output, Access::ReadWrite, operand);
                }
                asm::Unary::Call { arguments, returns } => {
                    self.transfer_unary(output, Access::Read, operand);

                    for argument in abi::ARGUMENT.iter().take(*arguments).copied() {
                        self.transfer_unary(
                            output,
                            Access::Read,
                            &operand::Unary::R(Temporary::Register(argument)),
                        );
                    }

                    // Clobber caller-saved registers
                    for register in abi::CALLER_SAVED.iter().copied() {
                        output.insert(Location::Register(register), Value::Conflict);
                    }

                    for r#return in abi::RETURN.iter().take(*returns).copied() {
                        self.transfer_unary(
                            output,
                            Access::Write,
                            &operand::Unary::R(Temporary::Register(r#return)),
                        );
                    }
                }
                asm::Unary::Hul | asm::Unary::Mod | asm::Unary::Div => {
                    self.transfer_unary(
                        output,
                        Access::Read,
                        &operand::Unary::R(Temporary::Register(Register::Rax)),
                    );

                    self.transfer_unary(
                        output,
                        Access::Read,
                        &operand::Unary::R(Temporary::Register(Register::Rdx)),
                    );

                    let (written, clobbered) = match unary {
                        asm::Unary::Div => (Register::Rax, Register::Rdx),
                        asm::Unary::Hul | asm::Unary::Mod => (Register::Rdx, Register::Rax),
                        _ => unreachable!(),
                    };

                    // Clobber `rax`
                    output.insert(Location::Register(clobbered), Value::Conflict);

                    self.transfer_unary(
                        output,
                        Access::Write,
                        &operand::Unary::R(Temporary::Register(written)),
                    );
                }
            },
            asm::Statement::Nullary(nullary) => match nullary {
                asm::Nullary::Nop => todo!(),
                asm::Nullary::Cqo => todo!(),
                asm::Nullary::Ret(_) => todo!(),
            },
        }
    }

    fn merge<'a, I>(&self, mut outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        input.clear();
        input.extend(outputs.next().into_iter().flatten().cloned().flatten());

        for output in outputs {
            let output = match output {
                Some(output) => output,
                None => {
                    input.clear();
                    return;
                }
            };

            input.retain(|location, old| match output.get(location) {
                None => false,
                Some(Value::Unknown) => true,
                Some(new @ Value::Temporary(_)) if new == old => true,
                Some(new @ Value::Temporary(_)) if matches!(old, Value::Unknown) => {
                    *old = *new;
                    true
                }
                Some(Value::Temporary(_)) | Some(Value::Conflict) => {
                    // NOTE: can track conflicting temporaries for debugging?
                    *old = Value::Conflict;
                    true
                }
            });
        }
    }
}

enum Access {
    Read,
    ReadWrite,
    Write,
}

impl<const LINEAR: bool> ValidAllocation<LINEAR> {
    /// Note: for binary operators, the source is always `Access::Read`, but the
    /// destination may differ.
    fn transfer_binary(
        &self,
        output: &mut Map<Location, Value>,
        access: Access,
        operands: &operand::Binary<Temporary>,
    ) {
        self.transfer_unary(output, Access::Read, &operands.source());
        self.transfer_unary(output, access, &operands.destination().into());
    }

    fn transfer_unary(
        &self,
        output: &mut Map<Location, Value>,
        access: Access,
        operand: &operand::Unary<Temporary>,
    ) {
        // Read source
        match access {
            Access::Write => (),
            Access::Read | Access::ReadWrite => {
                operand.map(|temporary| self.read(output, temporary));
            }
        }

        // Write source
        match access {
            Access::Read => (),
            Access::ReadWrite | Access::Write => match operand {
                operand::Unary::I(_) | operand::Unary::M(_) => (),
                operand::Unary::R(temporary) => {
                    self.write(output, *temporary);
                }
            },
        }
    }

    fn read(&self, output: &Map<Location, Value>, temporary: &Temporary) {
        let location = self.get(temporary);
        match output.get(&location).copied().unwrap_or(Value::Unknown) {
            Value::Conflict => panic!("Reading {} from conflict", temporary),
            Value::Temporary(allocated) => assert_eq!(*temporary, allocated),
            Value::Unknown => (),
        }
    }

    fn write(&self, output: &mut Map<Location, Value>, temporary: Temporary) {
        let location = self.get(&temporary);
        output.insert(location, Value::Temporary(temporary));
    }

    fn get(&self, temporary: &Temporary) -> Location {
        self.allocated
            .get(temporary)
            .copied()
            .map(Location::Register)
            .or_else(|| self.spilled.get(temporary).copied().map(Location::Stack))
            .or(match temporary {
                Temporary::Register(register) => Some(Location::Register(*register)),
                Temporary::Fixed(_) | Temporary::Fresh(_, _) => None,
            })
            .unwrap_or_else(|| panic!("[INTERNAL ERROR]: unallocated temporary: {}", temporary))
    }
}
