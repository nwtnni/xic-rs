use std::io;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::abi;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand::Immediate;
use crate::data::symbol;
use crate::interpret::postorder;
use crate::interpret::Global;
use crate::interpret::Local;
use crate::interpret::Operand;
use crate::interpret::Postorder;
use crate::interpret::Value;

pub fn interpret_hir<'io, R: io::BufRead + 'io, W: io::Write + 'io>(
    unit: &hir::Unit,
    stdin: R,
    stdout: W,
) -> anyhow::Result<()> {
    let unit = unit.map(Postorder::traverse_hir);

    let mut global = Global::new(&unit.data, stdin, stdout);
    let mut local = Local::new(
        &unit,
        &symbol::intern_static(abi::XI_MAIN),
        &[Value::Integer(0)],
    );

    assert!(local.interpret_hir(&unit, &mut global)?.is_empty());

    Ok(())
}

impl<'a> Local<'a, postorder::Hir<'a>> {
    fn interpret_hir(
        &mut self,
        unit: &ir::Unit<Postorder<postorder::Hir<'a>>>,
        global: &mut Global,
    ) -> anyhow::Result<Vec<Value>> {
        loop {
            let statement = match self.step() {
                Some(statement) => statement,
                None => return Ok(Vec::new()),
            };

            match statement {
                postorder::Hir::Expression(expression) => {
                    self.interpret_expression(unit, global, expression)?
                }
                postorder::Hir::Statement(statement) => {
                    if let Some(returns) = self.interpret_statement(global, statement)? {
                        return Ok(returns);
                    }
                }
            }
        }
    }

    fn interpret_expression(
        &mut self,
        unit: &ir::Unit<Postorder<postorder::Hir<'a>>>,
        global: &mut Global,
        expression: &hir::Expression,
    ) -> anyhow::Result<()> {
        log::trace!("E> {}", expression);
        match expression {
            hir::Expression::Sequence(_, _) => unreachable!(),
            hir::Expression::Immediate(Immediate::Integer(integer)) => {
                self.push(Operand::Integer(*integer))
            }
            hir::Expression::Immediate(Immediate::Label(label)) => {
                self.push(Operand::Label(*label, 8))
            }
            hir::Expression::Temporary(temporary) => self.push(Operand::Temporary(*temporary)),
            hir::Expression::Argument(index) => {
                let argument = self.get_argument(*index).into_operand();
                self.push(argument);
            }
            hir::Expression::Return(index) => {
                let r#return = self.get_return(*index).into_operand();
                self.push(r#return);
            }
            hir::Expression::Memory(_) => {
                let address = self.pop(global);
                self.push(Operand::Memory(address));
            }
            hir::Expression::Binary(binary, _, _) => self.interpret_binary(global, binary),
            hir::Expression::Call(_, arguments, _) => {
                let arguments = self.pop_list(global, arguments.len());
                let name = self.pop_name(global);

                log::info!("Calling function {} with arguments {:?}", name, arguments);

                let returns = global
                    .interpret_library(name, &arguments)
                    .unwrap_or_else(|| {
                        Local::new(unit, &name, &arguments).interpret_hir(unit, global)
                    })
                    .with_context(|| anyhow!("Calling function {}", name))?;

                if let Some(r#return) = returns.first() {
                    self.push(r#return.into_operand());
                }

                self.insert_returns(&returns);
            }
        }

        Ok(())
    }

    fn interpret_statement(
        &mut self,
        global: &mut Global,
        statement: &hir::Statement,
    ) -> anyhow::Result<Option<Vec<Value>>> {
        log::trace!("S> {}", statement);
        match statement {
            hir::Statement::Expression(_) => unreachable!(),
            hir::Statement::Label(_) => unreachable!(),
            hir::Statement::Sequence(_) => unreachable!(),
            hir::Statement::Jump(label) => {
                self.interpret_jump(label);
            }
            hir::Statement::CJump {
                condition,
                left: _,
                right: _,
                r#true,
                r#false,
            } => {
                if self.interpret_condition(global, condition) {
                    self.interpret_jump(r#true);
                } else {
                    self.interpret_jump(r#false);
                }
            }
            hir::Statement::Move {
                destination: _,
                source: _,
            } => self.interpret_move(global),
            hir::Statement::Return(returns) => {
                return Ok(Some(self.pop_list(global, returns.len())));
            }
        }

        Ok(None)
    }
}
