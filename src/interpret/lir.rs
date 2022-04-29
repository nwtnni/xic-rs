use std::io;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::constants;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::sexp::Serialize as _;
use crate::data::symbol;
use crate::interpret::postorder;
use crate::interpret::postorder::Postorder;
use crate::interpret::Global;
use crate::interpret::Local;
use crate::interpret::Operand;
use crate::interpret::Value;

pub fn interpret_lir<'io, R, W, T>(
    unit: &ir::Unit<lir::Function<T>>,
    stdin: R,
    stdout: W,
) -> anyhow::Result<()>
where
    R: io::BufRead + 'io,
    W: io::Write + 'io,
    T: lir::Target,
{
    let unit = Postorder::traverse_lir_unit(unit);

    let mut global = Global::new(&unit.data, stdin, stdout);
    let mut local = Local::new(
        &unit,
        &symbol::intern_static(constants::XI_MAIN),
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
            let instruction = match self.step() {
                Some(instruction) => instruction,
                None => return Ok(Vec::new()),
            };

            match instruction {
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
        log::trace!("E> {}", expression.sexp());
        match expression {
            lir::Expression::Immediate(operand::Immediate::Integer(integer)) => {
                self.push(Operand::Integer(*integer))
            }
            lir::Expression::Immediate(operand::Immediate::Label(label)) => {
                self.push(Operand::Label(*label, 8))
            }
            lir::Expression::Temporary(temporary) => self.push(Operand::Temporary(*temporary)),
            lir::Expression::Argument(index) => {
                let argument = self.get_argument(*index).into_operand();
                self.push(argument);
            }
            lir::Expression::Return(index) => {
                let r#return = self.get_return(*index).into_operand();
                self.push(r#return);
            }
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
        log::debug!("S> {}", statement.sexp());
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
                } else if let Some(label) = r#false.label() {
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

                self.insert_returns(&returns);
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
