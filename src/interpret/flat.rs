use std::collections::BTreeMap;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;

pub struct Flat<T> {
    instructions: Vec<T>,
    labels: BTreeMap<operand::Label, usize>,
}

impl<T> Flat<T> {
    pub fn get_instruction(&self, index: usize) -> Option<&T> {
        self.instructions.get(index)
    }

    pub fn get_label(&self, label: &operand::Label) -> Option<usize> {
        self.labels.get(label).copied()
    }
}

impl<T> Default for Flat<T> {
    fn default() -> Self {
        Self {
            instructions: Vec::new(),
            labels: BTreeMap::new(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Hir<'a> {
    Expression(&'a hir::Expression),
    Statement(&'a hir::Statement),
}

impl<'a> Flat<Hir<'a>> {
    pub fn flatten_hir_unit(unit: &'a ir::Unit<hir::Function>) -> ir::Unit<Flat<Hir<'a>>> {
        unit.map(Self::flatten_hir_function)
    }

    fn flatten_hir_function(function: &'a hir::Function) -> Flat<Hir<'a>> {
        let mut flat = Flat::default();
        flat.flatten_hir_statement(&function.statements);
        flat
    }

    fn flatten_hir_expression(&mut self, expression: &'a hir::Expression) {
        match expression {
            hir::Expression::Integer(_)
            | hir::Expression::Label(_)
            | hir::Expression::Temporary(_) => (),
            hir::Expression::Memory(address) => {
                self.flatten_hir_expression(address);
            }
            hir::Expression::Binary(_, left, right) => {
                self.flatten_hir_expression(left);
                self.flatten_hir_expression(right);
            }
            hir::Expression::Call(call) => self.flatten_call(call),
            hir::Expression::Sequence(statement, expression) => {
                self.flatten_hir_statement(statement);
                self.flatten_hir_expression(expression);
                return;
            }
        }

        self.instructions.push(Hir::Expression(expression));
    }

    fn flatten_hir_statement(&mut self, statement: &'a hir::Statement) {
        match statement {
            hir::Statement::Jump(expression) => self.flatten_hir_expression(expression),
            hir::Statement::CJump(condition, _, _) => self.flatten_hir_expression(condition),
            hir::Statement::Label(label) => {
                self.labels.insert(*label, self.instructions.len());
                return;
            }
            hir::Statement::Call(call) => self.flatten_call(call),
            hir::Statement::Move(into, from) => {
                self.flatten_hir_expression(into);
                self.flatten_hir_expression(from);
            }
            hir::Statement::Return(returns) => {
                returns
                    .iter()
                    .for_each(|r#return| self.flatten_hir_expression(r#return));
            }
            hir::Statement::Sequence(statements) => {
                statements
                    .iter()
                    .for_each(|statement| self.flatten_hir_statement(statement));
                return;
            }
        }

        self.instructions.push(Hir::Statement(statement));
    }

    fn flatten_call(&mut self, hir::Call { name, arguments }: &'a hir::Call) {
        self.flatten_hir_expression(name);
        arguments
            .iter()
            .for_each(|argument| self.flatten_hir_expression(argument));
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum Lir<'a> {
    Expression(&'a lir::Expression),
    Statement(&'a lir::Statement),
}

#[allow(dead_code)]
impl<'a> Flat<Lir<'a>> {
    pub fn flatten_lir_unit(unit: &'a ir::Unit<lir::Function>) -> ir::Unit<Flat<Lir<'a>>> {
        unit.map(Self::flatten_lir_function)
    }

    fn flatten_lir_function(function: &'a lir::Function) -> Flat<Lir<'a>> {
        let mut flat = Flat::default();
        function
            .statements
            .iter()
            .for_each(|statement| flat.flatten_lir_statement(statement));
        flat
    }

    fn flatten_lir_expression(&mut self, expression: &'a lir::Expression) {
        match expression {
            lir::Expression::Integer(_)
            | lir::Expression::Label(_)
            | lir::Expression::Temporary(_) => (),
            lir::Expression::Memory(address) => {
                self.flatten_lir_expression(address);
            }
            lir::Expression::Binary(_, left, right) => {
                self.flatten_lir_expression(left);
                self.flatten_lir_expression(right);
            }
        }

        self.instructions.push(Lir::Expression(expression));
    }

    fn flatten_lir_statement(&mut self, statement: &'a lir::Statement) {
        match statement {
            lir::Statement::Jump(expression) => self.flatten_lir_expression(expression),
            lir::Statement::CJump(condition, _, _) => self.flatten_lir_expression(condition),
            lir::Statement::Label(label) => {
                self.labels.insert(*label, self.instructions.len());
                return;
            }
            lir::Statement::Call(name, arguments) => {
                self.flatten_lir_expression(name);
                arguments
                    .iter()
                    .for_each(|argument| self.flatten_lir_expression(argument));
            }
            lir::Statement::Move(into, from) => {
                self.flatten_lir_expression(into);
                self.flatten_lir_expression(from);
            }
            lir::Statement::Return(returns) => {
                returns
                    .iter()
                    .for_each(|r#return| self.flatten_lir_expression(r#return));
            }
        }

        self.instructions.push(Lir::Statement(statement));
    }
}
