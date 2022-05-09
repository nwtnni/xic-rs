#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;

fn rewrite(function: &lir::Function<lir::Fallthrough>) -> Rewritten {
    let mut rewriter = Rewriter {
        arguments: (0..function.arguments)
            .map(|_| Temporary::fresh("ARG_INLINE"))
            .collect::<Vec<_>>(),
        returns: (0..function.returns)
            .map(|_| Temporary::fresh("RET_INLINE"))
            .collect::<Vec<_>>(),
        rename_temporary: RefCell::default(),
        rename_label: RefCell::default(),
        rewritten: Vec::new(),
    };

    rewriter.rewrite_function(function);

    Rewritten {
        arguments: rewriter.arguments,
        returns: rewriter.returns,
        statements: rewriter.rewritten,
    }
}

struct Rewritten {
    arguments: Vec<Temporary>,
    returns: Vec<Temporary>,
    statements: Vec<lir::Statement<lir::Fallthrough>>,
}

struct Rewriter {
    arguments: Vec<Temporary>,
    returns: Vec<Temporary>,
    rename_temporary: RefCell<BTreeMap<Temporary, Temporary>>,
    rename_label: RefCell<BTreeMap<Label, Label>>,
    rewritten: Vec<lir::Statement<lir::Fallthrough>>,
}

impl Rewriter {
    fn rewrite_function(&mut self, function: &lir::Function<lir::Fallthrough>) {
        for statement in &function.statements {
            self.rewrite_statement(statement);
        }
    }

    fn rewrite_statement(&mut self, statement: &lir::Statement<lir::Fallthrough>) {
        let statement = match statement {
            // Special case: argument passing. IR construction guarantees that these
            // will only be used as a source expression at the beginning of the function
            // to receive arguments.
            lir::Statement::Move {
                destination,
                source: lir::Expression::Argument(index),
            } => lir::Statement::Move {
                destination: self.rewrite_expression(destination),
                source: lir::Expression::Temporary(self.arguments[*index]),
            },

            // Special case: return passing. Note that CFG construction guarantees this will
            // be followed by a `jmp exit`, so we don't need to push a return statement--just
            // move the return values into the expected temporaries.
            lir::Statement::Return(returns) => {
                for (destination, source) in self.returns.iter().zip(returns) {
                    let r#move = lir::Statement::Move {
                        destination: lir::Expression::Temporary(*destination),
                        source: self.rewrite_expression(source),
                    };

                    self.rewritten.push(r#move);
                }

                return;
            }

            lir::Statement::Jump(label) => lir::Statement::Jump(self.rewrite_label(label)),
            lir::Statement::CJump {
                condition,
                left,
                right,
                r#true,
                r#false: lir::Fallthrough,
            } => lir::Statement::CJump {
                condition: *condition,
                left: self.rewrite_expression(left),
                right: self.rewrite_expression(right),
                r#true: self.rewrite_label(r#true),
                r#false: lir::Fallthrough,
            },
            lir::Statement::Call(function, arguments, returns) => {
                let function = self.rewrite_expression(function);
                let arguments = arguments
                    .iter()
                    .map(|argument| self.rewrite_expression(argument))
                    .collect();
                lir::Statement::Call(function, arguments, *returns)
            }
            lir::Statement::Label(label) => lir::Statement::Label(self.rewrite_label(label)),
            lir::Statement::Move {
                destination,
                source,
            } => lir::Statement::Move {
                destination: self.rewrite_expression(destination),
                source: self.rewrite_expression(source),
            },
        };

        self.rewritten.push(statement);
    }

    fn rewrite_expression(&self, expression: &lir::Expression) -> lir::Expression {
        match expression {
            lir::Expression::Argument(index) => lir::Expression::Argument(*index),
            lir::Expression::Return(index) => lir::Expression::Return(*index),
            lir::Expression::Immediate(immediate) => {
                lir::Expression::Immediate(self.rewrite_immediate(immediate))
            }
            lir::Expression::Temporary(temporary) => {
                lir::Expression::Temporary(self.rewrite_temporary(temporary))
            }
            lir::Expression::Memory(memory) => {
                lir::Expression::Memory(Box::new(self.rewrite_expression(memory)))
            }
            lir::Expression::Binary(binary, left, right) => lir::Expression::Binary(
                *binary,
                Box::new(self.rewrite_expression(left)),
                Box::new(self.rewrite_expression(right)),
            ),
        }
    }

    fn rewrite_immediate(&self, immediate: &Immediate) -> Immediate {
        match immediate {
            Immediate::Integer(integer) => Immediate::Integer(*integer),
            Immediate::Label(label) => Immediate::Label(self.rewrite_label(label)),
        }
    }

    fn rewrite_temporary(&self, temporary: &Temporary) -> Temporary {
        match temporary {
            Temporary::Register(register) => Temporary::Register(*register),
            // Note: fixed temporaries should only be generated in test cases, in which
            // case we _should_ provision a fresh name.
            Temporary::Fixed(symbol) => Temporary::fresh(symbol::resolve(*symbol)),
            Temporary::Fresh(symbol, index) => *self
                .rename_temporary
                .borrow_mut()
                .entry(Temporary::Fresh(*symbol, *index))
                .or_insert_with(|| Temporary::fresh(symbol::resolve(*symbol))),
        }
    }

    fn rewrite_label(&self, label: &Label) -> Label {
        match label {
            // Note: fixed labels are typically globally scoped, and should not be rewritten.
            Label::Fixed(fixed) => Label::Fixed(*fixed),
            Label::Fresh(symbol, index) => *self
                .rename_label
                .borrow_mut()
                .entry(Label::Fresh(*symbol, *index))
                .or_insert_with(|| Label::fresh(symbol::resolve(*symbol))),
        }
    }
}
