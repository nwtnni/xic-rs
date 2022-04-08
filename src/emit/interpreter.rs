#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::data::hir;
use crate::data::operand;

#[derive(Default)]
struct Flat<'a> {
    instructions: Vec<Hir<'a>>,
    labels: BTreeMap<operand::Label, usize>,
}

#[derive(Copy, Clone, Debug)]
enum Hir<'a> {
    Expression(&'a hir::Expression),
    Statement(&'a hir::Statement),
}

impl<'a> Flat<'a> {
    fn flatten_function(function: &'a hir::Function) -> Self {
        let mut flat = Flat::default();
        flat.flatten_statement(&function.statements);
        flat
    }

    fn flatten_expression(&mut self, expression: &'a hir::Expression) {
        match expression {
            hir::Expression::Integer(_)
            | hir::Expression::Label(_)
            | hir::Expression::Temporary(_) => (),
            hir::Expression::Memory(address) => {
                self.flatten_expression(address);
            }
            hir::Expression::Binary(_, left, right) => {
                self.flatten_expression(left);
                self.flatten_expression(right);
            }
            hir::Expression::Call(call) => self.flatten_call(call),
            hir::Expression::Sequence(statement, expression) => {
                self.flatten_statement(statement);
                self.flatten_expression(expression);
                return;
            }
        }

        self.instructions.push(Hir::Expression(expression));
    }

    fn flatten_statement(&mut self, statement: &'a hir::Statement) {
        match statement {
            hir::Statement::Jump(_) => (),
            hir::Statement::CJump(condition, _, _) => self.flatten_expression(condition),
            hir::Statement::Label(label) => {
                self.labels.insert(*label, self.instructions.len());
                return;
            }
            hir::Statement::Call(call) => self.flatten_call(call),
            hir::Statement::Move(into, from) => {
                self.flatten_expression(into);
                self.flatten_expression(from);
            }
            hir::Statement::Return(returns) => {
                returns
                    .iter()
                    .for_each(|r#return| self.flatten_expression(r#return));
            }
            hir::Statement::Sequence(statements) => {
                statements
                    .iter()
                    .for_each(|statement| self.flatten_statement(statement));
                return;
            }
        }

        self.instructions.push(Hir::Statement(statement));
    }

    fn flatten_call(&mut self, hir::Call { name, arguments }: &'a hir::Call) {
        self.flatten_expression(name);
        arguments
            .iter()
            .for_each(|argument| self.flatten_expression(argument));
    }
}
