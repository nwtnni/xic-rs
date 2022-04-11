use anyhow::anyhow;
use anyhow::Context as _;

use crate::data::hir;
use crate::data::ir;
use crate::data::operand;
use crate::interpret::global::Global;
use crate::interpret::global::Value;
use crate::interpret::local;
use crate::interpret::postorder;
use crate::interpret::postorder::PostorderHir;
use crate::util::symbol;

pub fn interpret_unit(unit: &ir::Unit<hir::Function>) -> anyhow::Result<()> {
    let unit = PostorderHir::traverse_hir_unit(unit);

    let mut global = Global::new();
    let mut local = local::Frame::new(&unit, &symbol::intern("_Imain_paai"), &[0]);

    debug_assert!(local.interpret_hir(&unit, &mut global)?.is_empty());

    Ok(())
}

impl<'a> local::Frame<'a, postorder::Hir<'a>> {
    fn interpret_hir(
        &mut self,
        unit: &ir::Unit<PostorderHir<'a>>,
        global: &mut Global,
    ) -> anyhow::Result<Vec<i64>> {
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
                    if let Some(r#returns) = self.interpret_statement(unit, global, statement)? {
                        return Ok(r#returns);
                    }
                }
            }
        }
    }

    fn interpret_expression(
        &mut self,
        unit: &ir::Unit<PostorderHir<'a>>,
        global: &mut Global,
        expression: &hir::Expression,
    ) -> anyhow::Result<()> {
        match expression {
            hir::Expression::Sequence(_, _) => unreachable!(),
            hir::Expression::Integer(integer) => self.push(Value::Integer(*integer)),
            hir::Expression::Label(label) => self.push(Value::Label(*label)),
            hir::Expression::Temporary(temporary) => self.push(Value::Temporary(*temporary)),
            hir::Expression::Memory(_) => {
                let address = self.pop_integer(global);
                self.push(Value::Memory(address));
            }
            hir::Expression::Binary(binary, _, _) => self.interpret_binary(global, binary),
            hir::Expression::Call(call) => {
                let mut r#return = self.interpret_call(unit, global, call)?;
                debug_assert_eq!(r#return.len(), 1);
                self.push(Value::Integer(r#return.remove(0)));
            }
        }

        Ok(())
    }

    fn interpret_statement(
        &mut self,
        unit: &ir::Unit<PostorderHir<'a>>,
        global: &mut Global,
        statement: &hir::Statement,
    ) -> anyhow::Result<Option<Vec<i64>>> {
        match statement {
            hir::Statement::Label(_) => unreachable!(),
            hir::Statement::Sequence(_) => unreachable!(),
            hir::Statement::Jump(_) => {
                self.interpret_jump();
                return Ok(None);
            }
            hir::Statement::CJump(_, r#true, r#false) => {
                self.interpret_cjump(global, r#true, r#false);
                return Ok(None);
            }
            hir::Statement::Call(call) => {
                for (index, r#return) in self
                    .interpret_call(unit, global, call)?
                    .into_iter()
                    .enumerate()
                {
                    self.insert(operand::Temporary::Return(index), r#return);
                }
            }
            hir::Statement::Move(_, _) => self.interpret_move(global),
            hir::Statement::Return(r#returns) => {
                return Ok(Some(self.pop_arguments(global, r#returns.len())));
            }
        }

        Ok(None)
    }

    fn interpret_call(
        &mut self,
        unit: &ir::Unit<PostorderHir<'a>>,
        global: &mut Global,
        call: &hir::Call,
    ) -> anyhow::Result<Vec<i64>> {
        let arguments = self.pop_arguments(global, call.arguments.len());
        let name = match self.pop_label() {
            operand::Label::Fixed(name) => name,
            operand::Label::Fresh(_, _) => panic!("calling fresh function name"),
        };

        global
            .interpret_library(name, &arguments)
            .unwrap_or_else(|| {
                local::Frame::new(unit, &name, &arguments).interpret_hir(unit, global)
            })
            .with_context(|| anyhow!("Calling function {}", name))
    }
}
