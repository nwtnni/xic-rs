#![allow(dead_code)]

use std::io;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::data::hir;
use crate::data::ir;
use crate::data::operand;
use crate::data::sexp::Serialize as _;
use crate::data::symbol;
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
        &symbol::intern_static("_Imain_paai"),
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
            let instruction = match self.step() {
                Some(instruction) => instruction,
                None => return Ok(Vec::new()),
            };

            match instruction {
                postorder::Hir::Expression(expression) => {
                    self.interpret_expression(unit, global, expression)?
                }
                postorder::Hir::Statement(statement) => {
                    if let Some(r#returns) = self.interpret_statement(global, statement)? {
                        return Ok(r#returns);
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
            hir::Expression::Integer(integer) => self.push(Operand::Integer(*integer)),
            hir::Expression::Label(label) => self.push(Operand::Label(*label, 8)),
            hir::Expression::Temporary(temporary) => self.push(Operand::Temporary(*temporary)),
            hir::Expression::Memory(_) => {
                let address = self.pop(global);
                self.push(Operand::Memory(address));
            }
            hir::Expression::Binary(binary, _, _) => self.interpret_binary(global, binary),
            hir::Expression::Call(_, arguments) => {
                let arguments = self.pop_arguments(global, arguments.len());
                let name = self.pop_name(global);

                log::info!("Calling function {} with arguments {:?}", name, arguments);

                let r#returns = global
                    .interpret_library(name, &arguments)
                    .unwrap_or_else(|| {
                        Local::new(unit, &name, &arguments).interpret_hir(unit, global)
                    })
                    .with_context(|| anyhow!("Calling function {}", name))?;

                if !r#returns.is_empty() {
                    self.push(r#returns[0].into_operand());
                }

                r#returns
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
    ) -> anyhow::Result<Option<Vec<Value>>> {
        log::trace!("S> {}", statement.sexp());
        match statement {
            hir::Statement::Expression(_) => unreachable!(),
            hir::Statement::Label(_) => unreachable!(),
            hir::Statement::Sequence(_) => unreachable!(),
            hir::Statement::Jump(_) => {
                self.interpret_jump(global);
                return Ok(None);
            }
            hir::Statement::CJump(_, r#true, r#false) => {
                self.interpret_cjump(global, r#true, r#false);
                return Ok(None);
            }
            hir::Statement::Move(_, _) => self.interpret_move(global),
            hir::Statement::Return(r#returns) => {
                return Ok(Some(self.pop_arguments(global, r#returns.len())));
            }
        }

        Ok(None)
    }
}
