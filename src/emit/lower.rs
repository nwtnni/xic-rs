use crate::abi;
use crate::data::hir;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::util;

pub fn emit_lir(function: &hir::Function) -> lir::Function<Label> {
    log::info!(
        "[{}] Emitting LIR for {}...",
        std::any::type_name::<hir::Function>(),
        function.name,
    );
    util::time!(
        "[{}] Done emitting LIR for {}",
        std::any::type_name::<hir::Function>(),
        function.name,
    );

    let mut lowerer = Lowerer::default();

    lowerer.lower_statement(&function.statement);
    let mut lowered = std::mem::take(&mut lowerer.lowered);

    match lowered.last() {
        Some(
            lir::Statement::Return(_) | lir::Statement::Jump(_) | lir::Statement::CJump { .. },
        ) => (),
        None
        | Some(
            lir::Statement::Call(_, _, _) | lir::Statement::Move { .. } | lir::Statement::Label(_),
        ) => {
            // Guaranteed valid by type-checker
            lowered.push(lir::Statement::Return(Vec::new()));
        }
    }

    lir::Function {
        name: function.name,
        statements: lowered,
        arguments: function.arguments.clone(),
        returns: function.returns,
        linkage: function.linkage,
        enter: (),
        exit: (),
    }
}

#[derive(Debug, Default)]
struct Lowerer {
    lowered: Vec<lir::Statement<Label>>,
}

impl Lowerer {
    fn lower_expression(&mut self, exp: &hir::Expression) -> lir::Expression {
        match exp {
            hir::Expression::Immediate(immediate) => lir::Expression::Immediate(*immediate),
            hir::Expression::Memory(memory) => {
                lir::Expression::Memory(Box::new(self.lower_expression(memory)))
            }
            hir::Expression::Temporary(temporary) => lir::Expression::Temporary(*temporary),
            hir::Expression::Sequence(statements, expression) => {
                self.lower_statement(statements);
                self.lower_expression(expression)
            }
            hir::Expression::Binary(binary, left, right) => {
                let (left, right) = self.lower_pair(left, right);
                lir::Expression::Binary(*binary, Box::new(left), Box::new(right))
            }
            hir::Expression::Call(function, arguments, returns) => {
                let function = if arguments.iter().all(|argument| commute(function, argument)) {
                    self.lower_expression(function)
                } else {
                    let save = Temporary::fresh("save");
                    let function = self.lower_expression(function);
                    self.lowered.push(lir::Statement::Move {
                        destination: lir::Expression::Temporary(save),
                        source: function,
                    });
                    lir::Expression::Temporary(save)
                };

                let arguments = self.lower_list(arguments);

                self.lowered
                    .push(lir::Statement::Call(function, arguments, returns.clone()));

                // Note: this return must not be used if `returns` is 0. This property
                // must be guaranteed when we emit HIR by wrapping any 0-return calls
                // in an `EXP` statement, to discard this temporary.
                lir::Expression::Temporary(match returns.first() {
                    None => Temporary::Fixed(symbol::intern_static("__INTERNAL_ERROR__")),
                    Some(r#return) => *r#return,
                })
            }
        }
    }

    fn lower_statement(&mut self, statement: &hir::Statement) {
        match statement {
            hir::Statement::Expression(expression) => {
                self.lower_expression(expression);
            }
            hir::Statement::Label(label) => self.lowered.push(lir::Statement::Label(*label)),
            hir::Statement::Sequence(statements) => {
                for statement in statements {
                    self.lower_statement(statement);
                }
            }
            hir::Statement::Jump(label) => self.lowered.push(lir::Statement::Jump(*label)),
            hir::Statement::CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => {
                let (left, right) = self.lower_pair(left, right);
                let cjump = lir::Statement::CJump {
                    condition: *condition,
                    left,
                    right,
                    r#true: *r#true,
                    r#false: *r#false,
                };
                self.lowered.push(cjump);
            }
            hir::Statement::Move {
                destination,
                source,
            } => match self.lower_expression(destination) {
                lir::Expression::Temporary(destination) => {
                    let source = self.lower_expression(source);
                    self.lowered.push(lir::Statement::Move {
                        destination: lir::Expression::Temporary(destination),
                        source,
                    });
                }
                lir::Expression::Memory(destination) if pure_expression(source) => {
                    let source = self.lower_expression(source);
                    self.lowered.push(lir::Statement::Move {
                        destination: lir::Expression::Memory(Box::new(*destination)),
                        source,
                    });
                }
                lir::Expression::Memory(destination) => {
                    let save = lir::Expression::Temporary(Temporary::fresh("save"));

                    self.lowered.push(lir::Statement::Move {
                        destination: save.clone(),
                        source: *destination,
                    });

                    let source = self.lower_expression(source);
                    self.lowered.push(lir::Statement::Move {
                        destination: lir::Expression::Memory(Box::new(save)),
                        source,
                    });
                }
                _ => unimplemented!(),
            },
            hir::Statement::Return(returns) => {
                let returns = self.lower_list(returns);
                self.lowered.push(lir::Statement::Return(returns));
            }
        }
    }

    fn lower_list(&mut self, expressions: &[hir::Expression]) -> Vec<lir::Expression> {
        if expressions.iter().all(pure_expression) {
            return expressions
                .iter()
                .map(|expression| self.lower_expression(expression))
                .collect();
        }

        expressions
            .iter()
            .map(|expression| {
                let save = lir::Expression::Temporary(Temporary::fresh("save"));
                let expression = self.lower_expression(expression);
                self.lowered.push(lir::Statement::Move {
                    destination: save.clone(),
                    source: expression,
                });
                save
            })
            .collect()
    }

    fn lower_pair(
        &mut self,
        left: &hir::Expression,
        right: &hir::Expression,
    ) -> (lir::Expression, lir::Expression) {
        if commute(left, right) {
            return (self.lower_expression(left), self.lower_expression(right));
        }

        let save = lir::Expression::Temporary(Temporary::fresh("save"));
        let left = self.lower_expression(left);

        self.lowered.push(lir::Statement::Move {
            destination: save.clone(),
            source: left,
        });

        let right = self.lower_expression(right);
        (save, right)
    }
}

fn commute(before: &hir::Expression, after: &hir::Expression) -> bool {
    match before {
        hir::Expression::Immediate(_) => true,
        hir::Expression::Binary(_, left, right) => commute(left, after) && commute(right, after),
        hir::Expression::Temporary(_)
        | hir::Expression::Memory(_)
        | hir::Expression::Call(_, _, _)
        | hir::Expression::Sequence(_, _) => pure_expression(after),
    }
}

fn pure_expression(expression: &hir::Expression) -> bool {
    match expression {
        hir::Expression::Immediate(_) | hir::Expression::Temporary(_) => true,
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
