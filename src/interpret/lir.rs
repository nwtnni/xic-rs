use std::io;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::constants;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::sexp::Serialize as _;
use crate::data::symbol;
use crate::interpret::local::Step;
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

    local.interpret_lir(&unit, &mut global)?;

    Ok(())
}

impl<'a, T: lir::Target> Local<'a, postorder::Lir<'a, T>> {
    fn interpret_lir(
        &mut self,
        unit: &ir::Unit<Postorder<postorder::Lir<'a, T>>>,
        global: &mut Global,
    ) -> anyhow::Result<()> {
        loop {
            let instruction = match self.step() {
                Some(instruction) => instruction,
                None => return Ok(()),
            };

            match instruction {
                postorder::Lir::Expression(expression) => {
                    self.interpret_expression(global, expression)?
                }
                postorder::Lir::Statement(statement) => {
                    if let Step::Return = self.interpret_statement(unit, global, statement)? {
                        return Ok(());
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
            lir::Expression::Immediate(operand::Immediate::Constant(integer)) => {
                self.push(Operand::Integer(*integer))
            }
            lir::Expression::Immediate(operand::Immediate::Label(label)) => {
                self.push(Operand::Label(*label, 8))
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
    ) -> anyhow::Result<Step> {
        log::debug!("S> {}", statement.sexp());
        match statement {
            lir::Statement::Label(_) => unreachable!(),
            lir::Statement::Jump(label) => {
                self.interpret_jump(label);
                return Ok(Step::Continue);
            }
            lir::Statement::CJump {
                condition: _,
                r#true,
                r#false,
            } => {
                if self.pop_boolean(global) {
                    self.interpret_jump(r#true);
                } else if let Some(label) = r#false.label() {
                    self.interpret_jump(label);
                }

                return Ok(Step::Continue);
            }
            lir::Statement::Call(_, arguments, returns) => {
                let arguments = self.pop_arguments(global, arguments.len());
                let name = self.pop_name(global);

                log::info!("Calling function {} with arguments {:?}", name, arguments);

                let returns = global
                    .interpret_library(name, &arguments)
                    .unwrap_or_else(|| {
                        let mut frame = Local::new(unit, &name, &arguments);
                        frame.interpret_lir(unit, global)?;
                        Ok(frame.pop_returns(*returns))
                    })
                    .with_context(|| anyhow!("Calling function {}", name))?;

                for (index, r#return) in returns.into_iter().enumerate() {
                    self.insert(operand::Temporary::Return(index), r#return);
                }
            }
            lir::Statement::Move {
                destination: _,
                source: _,
            } => self.interpret_move(global),
            lir::Statement::Return => {
                return Ok(Step::Return);
            }
        }

        Ok(Step::Continue)
    }
}
