use std::io;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::abi;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::interpret::postorder;
use crate::interpret::postorder::Postorder;
use crate::interpret::Global;
use crate::interpret::Local;
use crate::interpret::Operand;
use crate::interpret::Value;

pub fn interpret_lir<'io, R, W, T>(unit: &lir::Unit<T>, stdin: R, stdout: W) -> anyhow::Result<()>
where
    R: io::BufRead + 'io,
    W: io::Write + 'io,
    T: lir::Target,
{
    let unit = unit.map_ref(Postorder::traverse_lir);

    let mut global = Global::new(&unit.data, &unit.bss, stdin, stdout);

    let mut init = Local::new(&unit, &abi::mangle::init(), &[]);

    assert!(init.interpret_lir(&unit, &mut global)?.is_empty());

    let mut local = Local::new(
        &unit,
        &symbol::intern_static(abi::XI_MAIN),
        &[Value::Integer(0)],
    );

    assert!(local.interpret_lir(&unit, &mut global)?.is_empty());

    Ok(())
}

impl<'a, T: lir::Target> Local<'a, postorder::Lir<'a, T>> {
    fn interpret_lir(
        &mut self,
        unit: &ir::Unit<Postorder<postorder::Lir<'a, T>>>,
        global: &mut Global,
    ) -> anyhow::Result<Vec<Value>> {
        loop {
            let statement = match self.step() {
                Some(statement) => statement,
                None => return Ok(Vec::new()),
            };

            match statement {
                postorder::Lir::Expression(expression) => {
                    self.interpret_expression(global, expression)?
                }
                postorder::Lir::Statement(statement) => {
                    if let Some(returns) = self.interpret_statement(unit, global, statement)? {
                        return Ok(returns);
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
        log::trace!("E> {}", expression);
        match expression {
            lir::Expression::Immediate(Immediate::Integer(integer)) => {
                self.push(Operand::Integer(*integer))
            }
            lir::Expression::Immediate(Immediate::Label(label)) => {
                self.push(Operand::Label(*label, 0))
            }
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
        unit: &ir::Unit<Postorder<postorder::Lir<'a, T>>>,
        global: &mut Global,
        statement: &lir::Statement<T>,
    ) -> anyhow::Result<Option<Vec<Value>>> {
        log::debug!("S> {}", statement);
        match statement {
            lir::Statement::Label(_) => unreachable!(),
            lir::Statement::Jump(label) => {
                self.interpret_jump(label);
            }
            lir::Statement::CJump {
                condition,
                left: _,
                right: _,
                r#true,
                r#false,
            } => {
                if self.interpret_condition(global, condition) {
                    self.interpret_jump(r#true);
                } else if let Some(label) = r#false.target() {
                    self.interpret_jump(label);
                }
            }
            lir::Statement::Call(_, arguments, _) => {
                let arguments = self.pop_list(global, arguments.len());
                let name = self.pop_name(global);

                log::info!("Calling function {} with arguments {:?}", name, arguments);

                let returns = global
                    .interpret_library(name, &arguments)
                    .unwrap_or_else(|| {
                        Local::new(unit, &name, &arguments).interpret_lir(unit, global)
                    })
                    .with_context(|| anyhow!("Calling function {}", name))?;

                if let Some(r#return) = returns.first() {
                    self.push(r#return.into_operand());
                }

                for (index, r#return) in returns.into_iter().enumerate() {
                    self.insert(Temporary::Return(index), r#return);
                }
            }
            lir::Statement::Move {
                destination: _,
                source: _,
            } => self.interpret_move(global),
            lir::Statement::Return(returns) => {
                return Ok(Some(self.pop_list(global, returns.len())));
            }
        }

        Ok(None)
    }
}
