use anyhow::anyhow;

use crate::analyze::Analysis;
use crate::data::asm;
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
            asm::Statement::Binary(binary, operands) => match binary {
                asm::Binary::Add => todo!(),
                asm::Binary::Sub => todo!(),
                asm::Binary::Mul => todo!(),
                asm::Binary::And => todo!(),
                asm::Binary::Or => todo!(),
                asm::Binary::Xor => todo!(),
                asm::Binary::Cmp => todo!(),
                asm::Binary::Mov => todo!(),
                asm::Binary::Lea => todo!(),
                asm::Binary::Shl => todo!(),
            },
            asm::Statement::Unary(unary, operand) => match unary {
                asm::Unary::Push => todo!(),
                asm::Unary::Pop => todo!(),
                asm::Unary::Neg => todo!(),
                asm::Unary::Call { arguments, returns } => todo!(),
                asm::Unary::Hul => todo!(),
                asm::Unary::Div => todo!(),
                asm::Unary::Mod => todo!(),
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

impl<const LINEAR: bool> ValidAllocation<LINEAR> {
    fn get(&self, temporary: &Temporary) -> Location {
        self.allocated
            .get(temporary)
            .copied()
            .map(Location::Register)
            .or_else(|| self.spilled.get(temporary).copied().map(Location::Stack))
            .ok_or_else(|| anyhow!("[INTERNAL ERROR]: unallocated temporary: {}", temporary))
            .unwrap()
    }

    fn read(&self, output: &Map<Location, Value>, temporary: &Temporary) -> Value {
        let location = self.get(temporary);
        output.get(&location).copied().unwrap_or(Value::Unknown)
    }

    fn write(&mut self, output: &mut Map<Location, Value>, temporary: Temporary) {
        let location = self.get(&temporary);
        let value = output.entry(location).or_insert(Value::Unknown);
        *value = value.merge(&Value::Temporary(temporary));
    }
}
