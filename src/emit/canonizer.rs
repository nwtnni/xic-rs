use std::collections::BTreeMap;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;

#[derive(Debug, Default)]
pub struct Canonizer {
    canonized: Vec<lir::Statement>,
    purity: bool,
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
            Memory(memory) => self.canonize_expression(memory),
            Label(label) => lir::Expression::Label(*label),
            Temporary(temporary) => lir::Expression::Temporary(*temporary),
            Sequence(statements, expression) => {
                self.canonize_statement(statements);
                self.canonize_expression(expression)
            }
            Binary(binary, left, right) => {
                let left = self.canonize_expression(left);
                let index = self.canonized.len();
                let right = self.canonize_expression(right);
                if self.purity {
                    lir::Expression::Binary(*binary, Box::new(left), Box::new(right))
                } else {
                    let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                    let r#move = lir::Statement::Move(save.clone(), left);
                    self.canonized.insert(index, r#move);
                    lir::Expression::Binary(*binary, Box::new(save), Box::new(right))
                }
            }
            Call(call) => {
                self.canonize_call(call);

                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));

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
                let mut purity = true;
                for statement in statements {
                    self.purity = true;
                    self.canonize_statement(statement);
                    purity &= self.purity;
                }
                self.purity = purity;
            }
            Jump(expression) => {
                let jump = lir::Statement::Jump(self.canonize_expression(expression));
                self.canonized.push(jump);
                self.purity = false;
            }
            CJump(condition, r#true, r#false) => {
                let cjump =
                    lir::Statement::CJump(self.canonize_expression(condition), *r#true, *r#false);
                self.canonized.push(cjump);
                self.purity = false;
            }
            Move(hir::Expression::Memory(into), from) => {
                let into = self.canonize_expression(into);
                let index = self.canonized.len();
                let from = self.canonize_expression(from);
                if self.purity {
                    self.canonized.push(lir::Statement::Move(
                        lir::Expression::Memory(Box::new(into)),
                        from,
                    ));
                } else {
                    let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                    let r#move = lir::Statement::Move(save.clone(), into);
                    self.canonized.insert(index, r#move);
                    self.canonized.push(lir::Statement::Move(
                        lir::Expression::Memory(Box::new(save)),
                        from,
                    ));
                }
                self.purity = false;
            }
            Move(into, from) => {
                let into = self.canonize_expression(into);
                let index = self.canonized.len();
                let from = self.canonize_expression(from);
                if self.purity {
                    self.canonized.push(lir::Statement::Move(into, from));
                } else {
                    let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                    let into = lir::Statement::Move(save.clone(), into);
                    self.canonized.insert(index, into);
                    self.canonized.push(lir::Statement::Move(save, from));
                }
                self.purity = false;
            }
            Call(call) => self.canonize_call(call),
            Return(expressions) => {
                let mut purity = Vec::new();
                let mut canonized = Vec::new();
                let mut indices = Vec::new();

                for expression in expressions {
                    self.purity = true;
                    canonized.push(self.canonize_expression(expression));
                    indices.push(self.canonized.len());
                    purity.push(self.purity);
                }

                // Find last impure argument
                if let Some(impure) = purity.iter().rposition(|purity| !purity) {
                    // Move previous arguments into temps
                    let saved = (0..impure)
                        .map(|_| operand::Temporary::fresh("save"))
                        .collect::<Vec<_>>();

                    for index in (0..impure).rev() {
                        let save = lir::Expression::Temporary(saved[index]);
                        let into = lir::Statement::Move(save, canonized.remove(index));
                        self.canonized.insert(indices[index], into);
                    }

                    // Collect saved temps
                    let expressions = saved
                        .into_iter()
                        .map(lir::Expression::Temporary)
                        .chain(canonized.into_iter())
                        .collect::<Vec<_>>();

                    self.canonized.push(lir::Statement::Return(expressions));
                } else {
                    self.canonized.push(lir::Statement::Return(canonized));
                }

                // Does this matter?
                self.purity = true;
            }
        }
    }

    fn canonize_call(&mut self, call: &hir::Call) {
        let name = self.canonize_expression(&call.name);
        let index = self.canonized.len();

        let mut purity = Vec::new();
        let mut canonized = Vec::new();
        let mut indices = Vec::new();

        for arg in &call.arguments {
            self.purity = true;
            canonized.push(self.canonize_expression(arg));
            indices.push(self.canonized.len());
            purity.push(self.purity);
        }

        // Find last impure argument
        if let Some(impure) = purity.iter().rposition(|purity| !purity) {
            // Move previous arguments into temps
            let saved = (0..impure)
                .map(|_| operand::Temporary::fresh("save"))
                .collect::<Vec<_>>();

            for index in (0..impure).rev() {
                let save = lir::Expression::Temporary(saved[index]);
                let into = lir::Statement::Move(save, canonized.remove(index));
                self.canonized.insert(indices[index], into);
            }

            // Move function address into temp
            let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
            let into = lir::Statement::Move(save.clone(), name);
            self.canonized.insert(index, into);

            // Collect saved temps
            let args = saved
                .into_iter()
                .map(lir::Expression::Temporary)
                .chain(canonized.into_iter())
                .collect::<Vec<_>>();

            self.canonized.push(lir::Statement::Call(save, args));
        } else {
            self.canonized.push(lir::Statement::Call(name, canonized));
        }

        self.purity = false;
    }
}
