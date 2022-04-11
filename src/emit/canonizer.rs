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

    pub fn canonize_unit(mut self, unit: ir::Unit<hir::Function>) -> ir::Unit<lir::Function> {
        let mut functions = BTreeMap::default();
        for (name, function) in unit.functions {
            functions.insert(name, self.canonize_function(&function));
        }
        ir::Unit {
            name: unit.name,
            functions,
            data: unit.data,
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
            Binary(binary, left, right) => {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                let left = self.canonize_expression(left);

                self.canonized
                    .push(lir::Statement::Move(save.clone(), left));

                let right = self.canonize_expression(right);
                lir::Expression::Binary(*binary, Box::new(save), Box::new(right))
            }
            Call(call) => {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                self.canonize_call(call);

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
            Move(into, from) => {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                let into = self.canonize_expression(into);

                self.canonized
                    .push(lir::Statement::Move(save.clone(), into));

                let from = self.canonize_expression(from);
                self.canonized.push(lir::Statement::Move(save, from));
            }
            Call(call) => self.canonize_call(call),
            Return(r#returns) => {
                let r#returns = self.canonize_expressions(r#returns);
                self.canonized.push(lir::Statement::Return(r#returns));
            }
        }
    }

    fn canonize_call(&mut self, call: &hir::Call) {
        let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
        let name = self.canonize_expression(&call.name);

        self.canonized
            .push(lir::Statement::Move(save.clone(), name));

        let arguments = self.canonize_expressions(&call.arguments);
        self.canonized.push(lir::Statement::Call(save, arguments));
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
