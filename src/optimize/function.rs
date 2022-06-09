use std::cell::RefCell;
use std::iter;
use std::mem;

use crate::abi;
use crate::analyze::CallGraph;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::lir;
use crate::util;
use crate::util::Or;
use crate::Map;

pub fn inline_lir(lir: &mut lir::Unit<lir::Fallthrough>) {
    log::info!(
        "[{}] Inlining in {}...",
        std::any::type_name::<lir::Unit<lir::Fallthrough>>(),
        lir.name
    );
    util::time!(
        "[{}] Done inlining in {}",
        std::any::type_name::<lir::Unit<lir::Fallthrough>>(),
        lir.name
    );

    let call_graph = CallGraph::new(lir);

    let start = if lir
        .functions
        .contains_key(&symbol::intern_static(abi::XI_MAIN))
    {
        symbol::intern_static(abi::XI_MAIN)
    } else if let Some(name) = lir.functions.keys().next() {
        *name
    } else {
        return;
    };

    // We can only inline within our compilation unit.
    let postorder = call_graph
        .postorder(&start)
        .filter(|name| lir.functions.contains_key(name))
        .collect::<Vec<_>>();

    for name in &postorder {
        let mut function = lir.functions.remove(name).unwrap();

        function.statements = mem::take(&mut function.statements)
            .into_iter()
            .flat_map(|statement| match statement {
                lir::Statement::Call(
                    lir::Expression::Immediate(Immediate::Label(Label::Fixed(label))),
                    caller_arguments,
                    caller_returns,
                ) if lir.functions.contains_key(&label)
                    // Non-recursive
                    && !call_graph.is_recursive(&label)
                    && (
                        // Leaf function
                        call_graph.is_leaf(&label)

                        // Short function body
                        || lir.functions[&label].statements.len() < 30

                        // Constant arguments
                        || caller_arguments.iter().all(|expression| {
                            matches!(expression, lir::Expression::Immediate(_))
                    })
                    ) =>
                {
                    let Rewritten {
                        callee_arguments,
                        callee_returns,
                        statements,
                    } = rewrite(&lir.functions[&label]);

                    let arguments = callee_arguments
                        .into_iter()
                        .zip(caller_arguments)
                        .map(|(destination, source)| lir!((MOVE (TEMP destination) (TEMP source))));

                    let returns = (0..caller_returns)
                        .map(Temporary::Return)
                        .into_iter()
                        .zip(callee_returns)
                        .map(|(destination, source)| lir!((MOVE (TEMP destination) (TEMP source))));

                    Or::L(arguments.chain(statements).chain(returns))
                }
                statement => Or::R(iter::once(statement)),
            })
            .collect();

        lir.functions.insert(*name, function);
    }
}

fn rewrite(function: &lir::Function<lir::Fallthrough>) -> Rewritten {
    let mut rewriter = Rewriter {
        arguments: (0..function.arguments)
            .map(|_| Temporary::fresh("INLINE_ARG"))
            .collect::<Vec<_>>(),
        returns: (0..function.returns)
            .map(|_| Temporary::fresh("INLINE_RET"))
            .collect::<Vec<_>>(),
        rename_temporary: RefCell::default(),
        rename_label: RefCell::default(),
        rewritten: Vec::new(),
    };

    rewriter.rewrite_function(function);

    Rewritten {
        callee_arguments: rewriter.arguments,
        callee_returns: rewriter.returns,
        statements: rewriter.rewritten,
    }
}

struct Rewritten {
    callee_arguments: Vec<Temporary>,
    callee_returns: Vec<Temporary>,
    statements: Vec<lir::Statement<lir::Fallthrough>>,
}

struct Rewriter {
    arguments: Vec<Temporary>,
    returns: Vec<Temporary>,
    rename_temporary: RefCell<Map<Temporary, Temporary>>,
    rename_label: RefCell<Map<Label, Label>>,
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

            // Note: the argument and return cases are not symmetric because
            // arguments are passed from the caller and need to be rewritten,
            // while returns are received from callees and should be preserved.
            Temporary::Argument(index) => self.arguments[*index],
            Temporary::Return(index) => Temporary::Return(*index),

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
        // Note: assumes any fixed labels are globally visible and should not be renamed.
        let symbol = match label {
            Label::Fixed(symbol) => return Label::Fixed(*symbol),
            Label::Fresh(symbol, _) => *symbol,
        };

        *self
            .rename_label
            .borrow_mut()
            .entry(*label)
            .or_insert_with(|| Label::fresh(symbol::resolve(symbol)))
    }
}
