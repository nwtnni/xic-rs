use std::io;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::interpret::postorder;
use crate::interpret::postorder::Postorder;
use crate::interpret::Global;
use crate::interpret::Local;
use crate::interpret::Operand;
use crate::interpret::Value;
use crate::data::symbol;

pub fn interpret_lir<'io, R: io::BufRead + 'io, W: io::Write + 'io>(
    unit: &ir::Unit<lir::Function>,
    stdin: R,
    stdout: W,
) -> anyhow::Result<()> {
    let unit = Postorder::traverse_lir_unit(unit);

    let mut global = Global::new(&unit.data, stdin, stdout);
    let mut local = Local::new(
        &unit,
        &symbol::intern_static("_Imain_paai"),
        &[Value::Integer(0)],
    );

    assert!(local.interpret_lir(&unit, &mut global)?.is_empty());

    Ok(())
}

impl<'a> Local<'a, postorder::Lir<'a>> {
    fn interpret_lir(
        &mut self,
        unit: &ir::Unit<Postorder<postorder::Lir<'a>>>,
        global: &mut Global,
    ) -> anyhow::Result<Vec<Value>> {
        loop {
            let instruction = match self.step() {
                Some(instruction) => instruction,
                None => return Ok(Vec::new()),
            };

            match instruction {
                postorder::Lir::Expression(expression) => {
                    self.interpret_expression(global, expression)?
                }
                postorder::Lir::Statement(statement) => {
                    if let Some(r#returns) = self.interpret_statement(unit, global, statement)? {
                        return Ok(r#returns);
                    }
                }
            }
        }
    }

    fn interpret_expression(
        &mut self,
        global: &mut Global,
        expression: &lir::Expression,
    ) -> anyhow::Result<()> {
        match expression {
            lir::Expression::Integer(integer) => self.push(Operand::Integer(*integer)),
            lir::Expression::Label(label) => self.push(Operand::Label(*label, 8)),
            lir::Expression::Temporary(temporary) => self.push(Operand::Temporary(*temporary)),
            lir::Expression::Memory(_) => {
                let address = self.pop(global);
                self.push(Operand::Memory(address));
            }
            lir::Expression::Binary(binary, _, _) => self.interpret_binary(global, binary),
        }

        Ok(())
    }

    fn interpret_statement(
        &mut self,
        unit: &ir::Unit<Postorder<postorder::Lir<'a>>>,
        global: &mut Global,
        statement: &lir::Statement,
    ) -> anyhow::Result<Option<Vec<Value>>> {
        match statement {
            lir::Statement::Label(_) => unreachable!(),
            lir::Statement::Jump(_) => {
                self.interpret_jump(global);
                return Ok(None);
            }
            lir::Statement::CJump(_, r#true, r#false) => {
                self.interpret_cjump(global, r#true, r#false);
                return Ok(None);
            }
            lir::Statement::Call(_, arguments) => {
                let arguments = self.pop_arguments(global, arguments.len());
                let name = self.pop_name(global);

                let r#returns = global
                    .interpret_library(name, &arguments)
                    .unwrap_or_else(|| {
                        Local::new(unit, &name, &arguments).interpret_lir(unit, global)
                    })
                    .with_context(|| anyhow!("Calling function {}", name))?;

                for (index, r#return) in r#returns.into_iter().enumerate() {
                    self.insert(operand::Temporary::Return(index), r#return);
                }
            }
            lir::Statement::Move(_, _) => self.interpret_move(global),
            lir::Statement::Return(r#returns) => {
                return Ok(Some(self.pop_arguments(global, r#returns.len())));
            }
        }

        Ok(None)
    }
}
