use std::io;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::constants;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand;
use crate::data::sexp::Serialize as _;
use crate::data::symbol;
use crate::interpret::local::Step;
use crate::interpret::postorder;
use crate::interpret::Global;
use crate::interpret::Local;
use crate::interpret::Operand;
use crate::interpret::Postorder;
use crate::interpret::Value;

pub fn interpret_hir<'io, R: io::BufRead + 'io, W: io::Write + 'io>(
    unit: &ir::Unit<hir::Function>,
    stdin: R,
    stdout: W,
) -> anyhow::Result<()> {
    let unit = Postorder::traverse_hir_unit(unit);

    let mut global = Global::new(&unit.data, stdin, stdout);
    let mut local = Local::new(
        &unit,
        &symbol::intern_static(constants::XI_MAIN),
        &[Value::Integer(0)],
    );

    local.interpret_hir(&unit, &mut global)?;

    Ok(())
}

impl<'a> Local<'a, postorder::Hir<'a>> {
    fn interpret_hir(
        &mut self,
        unit: &ir::Unit<Postorder<postorder::Hir<'a>>>,
        global: &mut Global,
    ) -> anyhow::Result<()> {
        loop {
            let instruction = match self.step() {
                Some(instruction) => instruction,
                None => return Ok(()),
            };

            match instruction {
                postorder::Hir::Expression(expression) => {
                    self.interpret_expression(unit, global, expression)?
                }
                postorder::Hir::Statement(statement) => {
                    if let Step::Return = self.interpret_statement(global, statement)? {
                        return Ok(());
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
        log::trace!("E> {}", expression.sexp());
        match expression {
            hir::Expression::Sequence(_, _) => unreachable!(),
            hir::Expression::Immediate(operand::Immediate::Integer(integer)) => {
                self.push(Operand::Integer(*integer))
            }
            hir::Expression::Immediate(operand::Immediate::Label(label)) => {
                self.push(Operand::Label(*label, 8))
            }
            hir::Expression::Temporary(temporary) => self.push(Operand::Temporary(*temporary)),
            hir::Expression::Memory(_) => {
                let address = self.pop(global);
                self.push(Operand::Memory(address));
            }
            hir::Expression::Binary(binary, _, _) => self.interpret_binary(global, binary),
            hir::Expression::Call(_, arguments, returns) => {
                let arguments = self.pop_arguments(global, arguments.len());
                let name = self.pop_name(global);

                log::info!("Calling function {} with arguments {:?}", name, arguments);

                let returns = global
                    .interpret_library(name, &arguments)
                    .unwrap_or_else(|| {
                        let mut frame = Local::new(unit, &name, &arguments);
                        frame.interpret_hir(unit, global)?;
                        Ok(frame.pop_returns(*returns))
                    })
                    .with_context(|| anyhow!("Calling function {}", name))?;

                if let Some(r#return) = returns.first() {
                    self.push(r#return.into_operand());
                }

                returns
                    .into_iter()
                    .enumerate()
                    .for_each(|(index, r#return)| {
                        self.insert(operand::Temporary::Return(index), r#return)
                    });
            }
        }

        Ok(())
    }

    fn interpret_statement(
        &mut self,
        global: &mut Global,
        statement: &hir::Statement,
    ) -> anyhow::Result<Step> {
        log::trace!("S> {}", statement.sexp());
        match statement {
            hir::Statement::Expression(_) => unreachable!(),
            hir::Statement::Label(_) => unreachable!(),
            hir::Statement::Sequence(_) => unreachable!(),
            hir::Statement::Jump(label) => {
                self.interpret_jump(label);
                return Ok(Step::Continue);
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
                return Ok(Step::Continue);
            }
            hir::Statement::Move {
                destination: _,
                source: _,
            } => self.interpret_move(global),
            hir::Statement::Return => {
                return Ok(Step::Return);
            }
        }

        Ok(Step::Continue)
    }
}
