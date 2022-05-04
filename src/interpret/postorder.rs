//! This module linearizes the HIR and LIR data structures in post-order for interpretation.
//!
//! Linearization allows us to use a single integer index to track our position in the tree,
//! while also allowing arbitrary jumps between nodes in the tree. We use post-order so that
//! all children (e.g. of an `(ADD lhs rhs)` node) will be evaluated and pushed onto the stack
//! before evaluating the current node.
//!
//! Note that we store references to the original data structure to save space, instead of
//! duplicating subtrees.
//!
//! # Example
//!
//! ```text
//! (ESEQ
//!     (SEQ
//!         (LABEL a)
//!         (MOVE (TEMP t0) (CONST 0))
//!         (LABEL b)
//!         (MOVE (TEMP t0) (ADD (TEMP t0) (CONST 1)))
//!         (CJUMP (TEMP t0) a b))
//!     (TEMP t0))
//! ```
//!
//! ```text
//! a 00: (TEMP t0)
//!   01: (CONST 0)
//!   02: (MOVE (TEMP t0) (CONST 0))
//! b 03: (TEMP t0)
//!   04: (TEMP t0)
//!   05: (CONST 1)
//!   06: (ADD (TEMP t0) (CONST 1))
//!   07: (MOVE (TEMP t0) (ADD (TEMP t0) (CONST 1)))
//!   08: (TEMP t0)
//!   09: (CJUMP (TEMP t0) a b)
//!   10: (TEMP t0)
//! ```

use std::collections::BTreeMap;

use crate::data::hir;
use crate::data::lir;
use crate::data::operand::Label;

pub struct Postorder<T> {
    instructions: Vec<T>,
    labels: BTreeMap<Label, usize>,
}

impl<T> Postorder<T> {
    pub fn get_instruction(&self, index: usize) -> Option<&T> {
        self.instructions.get(index)
    }

    pub fn get_label(&self, label: &Label) -> Option<&usize> {
        self.labels.get(label)
    }
}

impl<T> Default for Postorder<T> {
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

impl<'a> Postorder<Hir<'a>> {
    pub fn traverse_hir(function: &'a hir::Function) -> Postorder<Hir<'a>> {
        let mut flat = Postorder::default();
        flat.traverse_hir_statement(&function.statement);
        flat
    }

    fn traverse_hir_expression(&mut self, expression: &'a hir::Expression) {
        match expression {
            hir::Expression::Immediate(_)
            | hir::Expression::Temporary(_)
            | hir::Expression::Argument(_)
            | hir::Expression::Return(_) => (),
            hir::Expression::Memory(address) => {
                self.traverse_hir_expression(address);
            }
            hir::Expression::Binary(_, left, right) => {
                self.traverse_hir_expression(left);
                self.traverse_hir_expression(right);
            }
            hir::Expression::Call(name, arguments, _) => {
                self.traverse_hir_expression(name);
                arguments
                    .iter()
                    .for_each(|argument| self.traverse_hir_expression(argument));
            }
            hir::Expression::Sequence(statement, expression) => {
                self.traverse_hir_statement(statement);
                self.traverse_hir_expression(expression);
                return;
            }
        }

        self.instructions.push(Hir::Expression(expression));
    }

    fn traverse_hir_statement(&mut self, statement: &'a hir::Statement) {
        match statement {
            hir::Statement::Jump(_) => (),
            hir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                self.traverse_hir_expression(left);
                self.traverse_hir_expression(right);
            }
            hir::Statement::Label(label) => {
                self.labels.insert(*label, self.instructions.len());
                return;
            }
            hir::Statement::Expression(expression) => {
                self.traverse_hir_expression(expression);
                return;
            }
            hir::Statement::Move {
                destination,
                source,
            } => {
                self.traverse_hir_expression(destination);
                self.traverse_hir_expression(source);
            }
            hir::Statement::Return(returns) => returns
                .iter()
                .for_each(|r#return| self.traverse_hir_expression(r#return)),
            hir::Statement::Sequence(statements) => {
                statements
                    .iter()
                    .for_each(|statement| self.traverse_hir_statement(statement));
                return;
            }
        }

        self.instructions.push(Hir::Statement(statement));
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Lir<'a, T> {
    Expression(&'a lir::Expression),
    Statement(&'a lir::Statement<T>),
}

impl<'a, T: lir::Target> Postorder<Lir<'a, T>> {
    pub fn traverse_lir(function: &'a lir::Function<T>) -> Postorder<Lir<'a, T>> {
        let mut flat = Postorder::default();
        function
            .statements
            .iter()
            .for_each(|statement| flat.traverse_lir_statement(statement));
        flat
    }

    fn traverse_lir_expression(&mut self, expression: &'a lir::Expression) {
        match expression {
            lir::Expression::Immediate(_)
            | lir::Expression::Temporary(_)
            | lir::Expression::Argument(_)
            | lir::Expression::Return(_) => (),
            lir::Expression::Memory(address) => {
                self.traverse_lir_expression(address);
            }
            lir::Expression::Binary(_, left, right) => {
                self.traverse_lir_expression(left);
                self.traverse_lir_expression(right);
            }
        }

        self.instructions.push(Lir::Expression(expression));
    }

    fn traverse_lir_statement(&mut self, statement: &'a lir::Statement<T>) {
        match statement {
            lir::Statement::Jump(_) => (),
            lir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                self.traverse_lir_expression(left);
                self.traverse_lir_expression(right);
            }
            lir::Statement::Label(label) => {
                self.labels.insert(*label, self.instructions.len());
                return;
            }
            lir::Statement::Call(name, arguments, _) => {
                self.traverse_lir_expression(name);
                arguments
                    .iter()
                    .for_each(|argument| self.traverse_lir_expression(argument));
            }
            lir::Statement::Move {
                destination,
                source,
            } => {
                self.traverse_lir_expression(destination);
                self.traverse_lir_expression(source);
            }
            lir::Statement::Return(returns) => returns
                .iter()
                .for_each(|r#return| self.traverse_lir_expression(r#return)),
        }

        self.instructions.push(Lir::Statement(statement));
    }
}
