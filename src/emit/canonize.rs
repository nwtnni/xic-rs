use std::collections::BTreeMap;

use crate::abi;
use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::symbol;

#[derive(Debug, Default)]
pub struct Canonizer {
    canonized: Vec<lir::Statement<lir::Label>>,
}

impl Canonizer {
    pub fn new() -> Self {
        Canonizer::default()
    }

    pub fn canonize_unit(
        mut self,
        unit: &ir::Unit<hir::Function>,
    ) -> ir::Unit<lir::Function<lir::Label>> {
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

    fn canonize_function(&mut self, function: &hir::Function) -> lir::Function<lir::Label> {
        self.canonize_statement(&function.statements);
        let mut canonized = std::mem::take(&mut self.canonized);

        match canonized.last() {
            None => unreachable!(),
            Some(
                lir::Statement::Return(_) | lir::Statement::Jump(_) | lir::Statement::CJump { .. },
            ) => (),
            Some(
                lir::Statement::Call(_, _, _)
                | lir::Statement::Move { .. }
                | lir::Statement::Label(_),
            ) => {
                // Guaranteed valid by type-checker
                canonized.push(lir::Statement::Return(Vec::new()));
            }
        }

        lir::Function {
            name: function.name,
            statements: canonized,
        }
    }

    fn canonize_expression(&mut self, exp: &hir::Expression) -> lir::Expression {
        use hir::Expression::*;
        match exp {
            Immediate(immediate) => lir::Expression::Immediate(*immediate),
            Argument(index) => lir::Expression::Argument(*index),
            Return(index) => lir::Expression::Return(*index),
            Memory(memory) => lir::Expression::Memory(Box::new(self.canonize_expression(memory))),
            Temporary(temporary) => lir::Expression::Temporary(*temporary),
            Sequence(statements, expression) => {
                self.canonize_statement(statements);
                self.canonize_expression(expression)
            }
            Binary(binary, left, right) => {
                let (left, right) = self.canonize_pair(left, right);
                lir::Expression::Binary(*binary, Box::new(left), Box::new(right))
            }
            Call(name, arguments, 1) => {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                let name = match &**name {
                    hir::Expression::Immediate(operand::Immediate::Label(name)) => name,
                    _ => unimplemented!("Calls to arbitrary expressions not yet implemented"),
                };

                let arguments = self.canonize_list(arguments);
                self.canonized.push(lir::Statement::Call(
                    lir::Expression::Immediate(operand::Immediate::Label(*name)),
                    arguments,
                    1,
                ));

                self.canonized.push(lir::Statement::Move {
                    destination: save.clone(),
                    source: lir::Expression::Return(0),
                });

                save
            }
            // 0- and multiple-return calls should be in (EXP (CALL ...)) statements.
            Call(_, _, _) => unreachable!("[TYPE ERROR]"),
        }
    }

    fn canonize_statement(&mut self, statement: &hir::Statement) {
        use hir::Statement::*;
        match statement {
            // Single-return calls should be in (MOVE (...) (CALL ...)) statements.
            Expression(hir::Expression::Call(_, _, 1)) => {
                unreachable!("[TYPE ERROR]")
            }
            Expression(hir::Expression::Call(name, arguments, _)) => {
                let name = match &**name {
                    hir::Expression::Immediate(operand::Immediate::Label(name)) => name,
                    _ => unimplemented!("Calls to arbitrary expressions not yet implemented"),
                };

                let arguments = self.canonize_list(arguments);
                self.canonized.push(lir::Statement::Call(
                    lir::Expression::Immediate(operand::Immediate::Label(*name)),
                    arguments,
                    0,
                ));
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
            Jump(label) => self.canonized.push(lir::Statement::Jump(*label)),
            CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => {
                let (left, right) = self.canonize_pair(left, right);
                let cjump = lir::Statement::CJump {
                    condition: *condition,
                    left,
                    right,
                    r#true: *r#true,
                    r#false: lir::Label(*r#false),
                };
                self.canonized.push(cjump);
            }
            Move {
                destination,
                source,
            } => match self.canonize_expression(destination) {
                lir::Expression::Temporary(destination) => {
                    let source = self.canonize_expression(source);
                    self.canonized.push(lir::Statement::Move {
                        destination: lir::Expression::Temporary(destination),
                        source,
                    });
                }
                lir::Expression::Memory(destination) if pure_expression(source) => {
                    let source = self.canonize_expression(source);
                    self.canonized.push(lir::Statement::Move {
                        destination: lir::Expression::Memory(Box::new(*destination)),
                        source,
                    });
                }
                lir::Expression::Memory(destination) => {
                    let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));

                    self.canonized.push(lir::Statement::Move {
                        destination: save.clone(),
                        source: *destination,
                    });

                    let source = self.canonize_expression(source);
                    self.canonized.push(lir::Statement::Move {
                        destination: lir::Expression::Memory(Box::new(save)),
                        source,
                    });
                }
                _ => unimplemented!(),
            },
            Return(returns) => {
                let returns = self.canonize_list(returns);
                self.canonized.push(lir::Statement::Return(returns));
            }
        }
    }

    fn canonize_list(&mut self, expressions: &[hir::Expression]) -> Vec<lir::Expression> {
        if expressions.iter().all(pure_expression) {
            return expressions
                .iter()
                .map(|expression| self.canonize_expression(expression))
                .collect();
        }

        expressions
            .iter()
            .map(|expression| {
                let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
                let expression = self.canonize_expression(expression);
                self.canonized.push(lir::Statement::Move {
                    destination: save.clone(),
                    source: expression,
                });
                save
            })
            .collect()
    }

    fn canonize_pair(
        &mut self,
        left: &hir::Expression,
        right: &hir::Expression,
    ) -> (lir::Expression, lir::Expression) {
        if commute(left, right) {
            return (
                self.canonize_expression(left),
                self.canonize_expression(right),
            );
        }

        let save = lir::Expression::Temporary(operand::Temporary::fresh("save"));
        let left = self.canonize_expression(left);

        self.canonized.push(lir::Statement::Move {
            destination: save.clone(),
            source: left,
        });

        let right = self.canonize_expression(right);
        (save, right)
    }
}

fn commute(before: &hir::Expression, after: &hir::Expression) -> bool {
    use hir::Expression::*;
    match before {
        Immediate(operand::Immediate::Integer(_)) | Argument(_) => true,
        Binary(_, left, right) => commute(left, after) && commute(right, after),
        Immediate(operand::Immediate::Label(_))
        | Temporary(_)
        | Return(_)
        | Memory(_)
        | Call(_, _, _)
        | Sequence(_, _) => pure_expression(after),
    }
}

fn pure_expression(expression: &hir::Expression) -> bool {
    use hir::Expression::*;
    match expression {
        Immediate(operand::Immediate::Integer(_)) | Temporary(_) | Argument(_) | Return(_) => true,
        Immediate(operand::Immediate::Label(_)) => false,
        Memory(expression) => pure_expression(expression),
        Binary(_, left, right) => pure_expression(left) && pure_expression(right),
        Sequence(statement, expression) => pure_statement(statement) && pure_expression(expression),
        Call(name, _, _) => {
            let name = match &**name {
                Immediate(operand::Immediate::Label(operand::Label::Fixed(name))) => {
                    symbol::resolve(*name)
                }
                _ => return false,
            };

            // Specialize standard library functions
            matches!(
                name,
                abi::XI_ALLOC
                    | abi::XI_PRINT
                    | abi::XI_PRINTLN
                    | abi::XI_READLN
                    | abi::XI_GETCHAR
                    | abi::XI_EOF
                    | abi::XI_UNPARSE_INT
                    | abi::XI_PARSE_INT,
            )
        }
    }
}

fn pure_statement(statement: &hir::Statement) -> bool {
    match statement {
        hir::Statement::Jump(_)
        | hir::Statement::CJump { .. }
        | hir::Statement::Move { .. }
        | hir::Statement::Return(_) => false,
        hir::Statement::Label(_) => true,
        hir::Statement::Expression(expression) => pure_expression(expression),
        hir::Statement::Sequence(statements) => statements.iter().all(pure_statement),
    }
}
