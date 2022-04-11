use std::collections::BTreeMap;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::data::hir;
use crate::data::ir;
use crate::data::operand;
use crate::interpret::global::Global;
use crate::interpret::global::Value;
use crate::interpret::postorder;
use crate::interpret::postorder::PostorderHir;
use crate::util::symbol;
use crate::util::symbol::Symbol;

struct Local<'a> {
    postorder: &'a PostorderHir<'a>,
    index: usize,
    temporaries: BTreeMap<operand::Temporary, i64>,
    stack: Vec<Value>,
}

pub fn interpret_unit(unit: &ir::Unit<hir::Function>) -> anyhow::Result<()> {
    let unit = PostorderHir::traverse_hir_unit(unit);

    let mut global = Global::new();
    let mut local = Local::new(&unit, &symbol::intern("_Imain_paai"), &[0]);

    debug_assert!(local.run(&unit, &mut global)?.is_empty());

    Ok(())
}

impl<'a> Local<'a> {
    fn new(unit: &'a ir::Unit<PostorderHir<'a>>, name: &Symbol, arguments: &[i64]) -> Self {
        let postorder = unit.functions.get(name).unwrap();

        let mut temporaries = BTreeMap::new();

        for (index, argument) in arguments.iter().copied().enumerate() {
            temporaries.insert(operand::Temporary::Argument(index), argument);
        }

        Local {
            postorder,
            index: 0,
            temporaries,
            stack: Vec::new(),
        }
    }

    fn run(
        &mut self,
        unit: &ir::Unit<PostorderHir<'a>>,
        global: &mut Global,
    ) -> anyhow::Result<Vec<i64>> {
        loop {
            let instruction = match self.postorder.get_instruction(self.index) {
                Some(instruction) => instruction,
                None => return Ok(Vec::new()),
            };

            match instruction {
                postorder::Hir::Expression(expression) => {
                    self.interpret_expression(unit, global, expression)?;
                    self.index += 1;
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
            hir::Expression::Integer(integer) => self.stack.push(Value::Integer(*integer)),
            hir::Expression::Label(label) => self.stack.push(Value::Label(*label)),
            hir::Expression::Temporary(temporary) => self.stack.push(Value::Temporary(*temporary)),
            hir::Expression::Memory(_) => {
                let address = self.pop_integer(global);
                self.stack.push(Value::Memory(address));
            }
            hir::Expression::Binary(binary, _, _) => {
                let right = self.pop_integer(global);
                let left = self.pop_integer(global);
                let value = match binary {
                    ir::Binary::Add => left.wrapping_add(right),
                    ir::Binary::Sub => left.wrapping_sub(right),
                    ir::Binary::Mul => left.wrapping_mul(right),
                    ir::Binary::Hul => (((left as i128) * (right as i128)) >> 64) as i64,
                    ir::Binary::Div => left / right,
                    ir::Binary::Mod => left % right,
                    ir::Binary::Xor => left ^ right,
                    ir::Binary::Ls => left << right,
                    ir::Binary::Rs => ((left as u64) >> right) as i64,
                    ir::Binary::ARs => left >> right,
                    ir::Binary::Lt => (left < right) as bool as i64,
                    ir::Binary::Le => (left <= right) as bool as i64,
                    ir::Binary::Ge => (left >= right) as bool as i64,
                    ir::Binary::Gt => (left > right) as bool as i64,
                    ir::Binary::Ne => (left != right) as bool as i64,
                    ir::Binary::Eq => (left == right) as bool as i64,
                    ir::Binary::And => {
                        debug_assert!(left == 0 || left == 1);
                        debug_assert!(right == 0 || right == 1);
                        left & right
                    }
                    ir::Binary::Or => {
                        debug_assert!(left == 0 || left == 1);
                        debug_assert!(right == 0 || right == 1);
                        left | right
                    }
                };
                self.stack.push(Value::Integer(value));
            }
            hir::Expression::Call(call) => {
                let mut r#return = self.interpret_call(unit, global, call)?;
                debug_assert_eq!(r#return.len(), 1);
                self.stack.push(Value::Integer(r#return.remove(0)));
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
                let label = self.pop_label();
                self.index = self.postorder.get_label(&label).unwrap();
                return Ok(None);
            }
            hir::Statement::CJump(_, r#true, r#false) => {
                let label = match self.pop_integer(global) {
                    0 => r#false,
                    1 => r#true,
                    _ => unreachable!(),
                };
                self.index = self.postorder.get_label(label).unwrap();
                return Ok(None);
            }
            hir::Statement::Call(call) => {
                for (index, r#return) in self
                    .interpret_call(unit, global, call)?
                    .into_iter()
                    .enumerate()
                {
                    self.temporaries
                        .insert(operand::Temporary::Return(index), r#return);
                }
            }
            hir::Statement::Move(_, _) => {
                let from = self.pop_integer(global);
                let into = self.stack.pop();
                match into {
                    None => panic!("empty stack"),
                    Some(Value::Integer(_)) => panic!("writing into integer"),
                    Some(Value::Memory(address)) => global.write(address, from),
                    Some(Value::Temporary(temporary)) => {
                        self.temporaries.insert(temporary, from);
                    }
                    Some(Value::Label(_)) => panic!("writing into label"),
                }
            }
            hir::Statement::Return(r#returns) => {
                let mut r#returns = (0..r#returns.len())
                    .map(|_| self.pop_integer(global))
                    .collect::<Vec<_>>();

                r#returns.reverse();

                return Ok(Some(r#returns));
            }
        }

        self.index += 1;
        Ok(None)
    }

    fn interpret_call(
        &mut self,
        unit: &ir::Unit<PostorderHir<'a>>,
        global: &mut Global,
        call: &hir::Call,
    ) -> anyhow::Result<Vec<i64>> {
        let mut arguments = (0..call.arguments.len())
            .map(|_| self.pop_integer(global))
            .collect::<Vec<_>>();

        arguments.reverse();

        let name = match self.pop_label() {
            operand::Label::Fixed(name) => name,
            operand::Label::Fresh(_, _) => panic!("calling fresh function name"),
        };

        global
            .interpret_library(name, &arguments)
            .unwrap_or_else(|| Local::new(unit, &name, &arguments).run(unit, global))
            .with_context(|| anyhow!("Calling function {}", name))
    }

    fn pop_integer(&mut self, global: &Global) -> i64 {
        match self.stack.pop() {
            None => panic!("empty stack"),
            Some(Value::Integer(integer)) => integer,
            Some(Value::Memory(address)) => global.read(address),
            Some(Value::Label(_)) => panic!("using label as integer"),
            Some(Value::Temporary(temporary)) => self.temporaries[&temporary],
        }
    }

    fn pop_label(&mut self) -> operand::Label {
        match self.stack.pop() {
            None => panic!("empty stack"),
            Some(Value::Integer(_)) => panic!("using integer as label"),
            Some(Value::Memory(_)) => panic!("using memory as label"),
            Some(Value::Label(label)) => label,
            Some(Value::Temporary(_)) => panic!("using temporary as label"),
        }
    }
}
