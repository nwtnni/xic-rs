use std::collections::BTreeMap;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;

#[derive(Debug, Default)]
pub struct Canonizer {
    canonized: Vec<lir::Statement>,
}

impl Canonizer {
    pub fn new() -> Self {
        Canonizer::default()
    }

    pub fn canonize_unit(mut self, unit: &ir::Unit<hir::Function>) -> ir::Unit<lir::Function> {
        let mut functions = BTreeMap::default();
        for (name, function) in &unit.functions {
            functions.insert(*name, self.canonize_function(function));
        }
        ir::Unit {
            name: unit.name,
            functions,
            data: unit.data.clone(),
        }
    }

    fn canonize_function(&mut self, function: &hir::Function) -> lir::Function {
        self.canonize_statement(&function.statements);
        let mut canonized = std::mem::take(&mut self.canonized);
        if let Some(lir::Statement::Return(_)) = canonized.last() {
        } else {
            canonized.push(lir::Statement::Return(vec![]));
        }
        lir::Function {
            name: function.name,
            statements: canonized,
        }
    }

    fn canonize_expression(&mut self, exp: &hir::Expression) -> lir::Expression {
        use hir::Expression::*;
        match exp {
            Integer(integer) => lir::Expression::Integer(*integer),
            Memory(memory) => lir::Expression::Memory(Box::new(self.canonize_expression(memory))),
            Label(label) => lir::Expression::Label(*label),
            Temporary(temporary) => lir::Expression::Temporary(*temporary),
            Sequence(statements, expression) => {
                self.canonize_statement(statements);
                self.canonize_expression(expression)
            }
            Binary(binary, left, right) if commute(left, right) => {
                let left = self.canonize_expression(left);
                let right = self.canonize_expression(right);
                lir::Expression::Binary(*binary, Box::new(left), Box::new(right))
            }
            Binary(binary, left, right) => {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                let left = self.canonize_expression(left);

                self.canonized
                    .push(lir::Statement::Move(save.clone(), left));

                let right = self.canonize_expression(right);
                lir::Expression::Binary(*binary, Box::new(save), Box::new(right))
            }
            Call(_, _, 0) => unreachable!("[TYPE ERROR]: procedure call"),
            Call(name, arguments, returns) => {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                let name = match &**name {
                    hir::Expression::Label(name) => name,
                    _ => unimplemented!("Calls to arbitrary expressions not yet implemented"),
                };

                let arguments = self.canonize_expressions(arguments);
                self.canonized.push(lir::Statement::Call(
                    lir::Expression::Label(*name),
                    arguments,
                    *returns,
                ));

                self.canonized.push(lir::Statement::Move(
                    save.clone(),
                    lir::Expression::Temporary(operand::Temporary::Return(0)),
                ));

                save
            }
        }
    }

    fn canonize_statement(&mut self, statement: &hir::Statement) {
        use hir::Statement::*;
        match statement {
            Expression(hir::Expression::Call(name, arguments, 0)) => {
                let name = match &**name {
                    hir::Expression::Label(name) => name,
                    _ => unimplemented!("Calls to arbitrary expressions not yet implemented"),
                };

                let arguments = self.canonize_expressions(arguments);
                self.canonized.push(lir::Statement::Call(
                    lir::Expression::Label(*name),
                    arguments,
                    0,
                ));
            }
            Expression(hir::Expression::Call(_, _, _)) => {
                unreachable!("[TYPE ERROR]: function call")
            }
            Expression(expression) => {
                self.canonize_expression(expression);
            }
            Label(label) => self.canonized.push(lir::Statement::Label(*label)),
            Sequence(statements) => {
                for statement in statements {
                    self.canonize_statement(statement);
                }
            }
            Jump(expression) => {
                let jump = lir::Statement::Jump(self.canonize_expression(expression));
                self.canonized.push(jump);
            }
            CJump(condition, r#true, r#false) => {
                let cjump =
                    lir::Statement::CJump(self.canonize_expression(condition), *r#true, *r#false);
                self.canonized.push(cjump);
            }
            Move(into, from) => match self.canonize_expression(into) {
                lir::Expression::Temporary(into) => {
                    let from = self.canonize_expression(from);
                    self.canonized
                        .push(lir::Statement::Move(lir::Expression::Temporary(into), from));
                }
                lir::Expression::Memory(into) if pure(from) => {
                    let from = self.canonize_expression(from);
                    self.canonized.push(lir::Statement::Move(
                        lir::Expression::Memory(Box::new(*into)),
                        from,
                    ));
                }
                lir::Expression::Memory(into) => {
                    let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));

                    self.canonized
                        .push(lir::Statement::Move(save.clone(), *into));

                    let from = self.canonize_expression(from);
                    self.canonized.push(lir::Statement::Move(
                        lir::Expression::Memory(Box::new(save)),
                        from,
                    ));
                }
                _ => unimplemented!(),
            },
            Return(r#returns) => {
                let r#returns = self.canonize_expressions(r#returns);
                self.canonized.push(lir::Statement::Return(r#returns));
            }
        }
    }

    fn canonize_expressions(&mut self, expressions: &[hir::Expression]) -> Vec<lir::Expression> {
        expressions
            .iter()
            .map(|expression| {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                let expression = self.canonize_expression(expression);
                self.canonized
                    .push(lir::Statement::Move(save.clone(), expression));
                save
            })
            .collect()
    }
}

fn commute(before: &hir::Expression, after: &hir::Expression) -> bool {
    use hir::Expression::*;
    match before {
        Integer(_) => true,
        Binary(_, left, right) => commute(left, after) && commute(right, after),
        Label(_) | Temporary(_) | Memory(_) | Call(_, _, _) | Sequence(_, _) => pure(after),
    }
}

fn pure(expression: &hir::Expression) -> bool {
    use hir::Expression::*;
    match expression {
        Integer(_) | Label(_) | Temporary(_) => true,
        Memory(expression) => pure(expression),
        Binary(_, left, right) => pure(left) && pure(right),
        Call(_, _, _) | Sequence(_, _) => false,
    }
}
