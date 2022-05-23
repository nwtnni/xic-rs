use crate::abi;
use crate::data::hir;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;

pub fn canonize_function(function: &hir::Function) -> lir::Function<Label> {
    let mut canonizer = Canonizer::default();

    canonizer.canonize_statement(&function.statement);
    let mut canonized = std::mem::take(&mut canonizer.canonized);

    match canonized.last() {
        None => unreachable!(),
        Some(
            lir::Statement::Return(_) | lir::Statement::Jump(_) | lir::Statement::CJump { .. },
        ) => (),
        Some(
            lir::Statement::Call(_, _, _) | lir::Statement::Move { .. } | lir::Statement::Label(_),
        ) => {
            // Guaranteed valid by type-checker
            canonized.push(lir::Statement::Return(Vec::new()));
        }
    }

    lir::Function {
        name: function.name,
        statements: canonized,
        arguments: function.arguments,
        returns: function.returns,
        enter: (),
        exit: (),
    }
}

#[derive(Debug, Default)]
struct Canonizer {
    canonized: Vec<lir::Statement<Label>>,
}

impl Canonizer {
    fn canonize_expression(&mut self, exp: &hir::Expression) -> lir::Expression {
        match exp {
            hir::Expression::Immediate(immediate) => lir::Expression::Immediate(*immediate),
            hir::Expression::Memory(memory) => {
                lir::Expression::Memory(Box::new(self.canonize_expression(memory)))
            }
            hir::Expression::Temporary(temporary) => lir::Expression::Temporary(*temporary),
            hir::Expression::Sequence(statements, expression) => {
                self.canonize_statement(statements);
                self.canonize_expression(expression)
            }
            hir::Expression::Binary(binary, left, right) => {
                let (left, right) = self.canonize_pair(left, right);
                lir::Expression::Binary(*binary, Box::new(left), Box::new(right))
            }
            hir::Expression::Call(function, arguments, returns) => {
                let function = if arguments.iter().all(|argument| commute(function, argument)) {
                    self.canonize_expression(function)
                } else {
                    let save = Temporary::fresh("save");
                    let function = self.canonize_expression(function);
                    self.canonized.push(lir::Statement::Move {
                        destination: lir::Expression::Temporary(save),
                        source: function,
                    });
                    lir::Expression::Temporary(save)
                };

                let arguments = self.canonize_list(arguments);

                self.canonized
                    .push(lir::Statement::Call(function, arguments, *returns));

                // Note: this return must not be used if `returns` is 0. This property
                // must be guaranteed when we emit HIR by wrapping any 0-return calls
                // in an `EXP` statement, to discard this temporary.
                lir::Expression::Temporary(Temporary::Return(0))
            }
        }
    }

    fn canonize_statement(&mut self, statement: &hir::Statement) {
        match statement {
            hir::Statement::Expression(expression) => {
                self.canonize_expression(expression);
            }
            hir::Statement::Label(label) => self.canonized.push(lir::Statement::Label(*label)),
            hir::Statement::Sequence(statements) => {
                for statement in statements {
                    self.canonize_statement(statement);
                }
            }
            hir::Statement::Jump(label) => self.canonized.push(lir::Statement::Jump(*label)),
            hir::Statement::CJump {
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
                    r#false: *r#false,
                };
                self.canonized.push(cjump);
            }
            hir::Statement::Move {
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
                    let save = lir::Expression::Temporary(Temporary::fresh("save"));

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
            hir::Statement::Return(returns) => {
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
                let save = lir::Expression::Temporary(Temporary::fresh("save"));
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

        let save = lir::Expression::Temporary(Temporary::fresh("save"));
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
    match before {
        hir::Expression::Immediate(Immediate::Integer(_)) => true,
        hir::Expression::Binary(_, left, right) => commute(left, after) && commute(right, after),
        hir::Expression::Immediate(Immediate::Label(_))
        | hir::Expression::Temporary(_)
        | hir::Expression::Memory(_)
        | hir::Expression::Call(_, _, _)
        | hir::Expression::Sequence(_, _) => pure_expression(after),
    }
}

fn pure_expression(expression: &hir::Expression) -> bool {
    match expression {
        hir::Expression::Immediate(Immediate::Integer(_)) | hir::Expression::Temporary(_) => true,
        hir::Expression::Immediate(Immediate::Label(_)) => false,
        hir::Expression::Memory(expression) => pure_expression(expression),
        hir::Expression::Binary(_, left, right) => pure_expression(left) && pure_expression(right),
        hir::Expression::Sequence(statement, expression) => {
            pure_statement(statement) && pure_expression(expression)
        }
        hir::Expression::Call(name, _, _) => {
            let name = match &**name {
                hir::Expression::Immediate(Immediate::Label(Label::Fixed(name))) => {
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
