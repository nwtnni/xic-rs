use std::borrow::Cow;
use std::cell::RefCell;
use std::iter;
use std::mem;

use crate::abi;
use crate::analyze::CallGraph;
use crate::cfg;
use crate::cfg::Cfg;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::lir;
use crate::util;
use crate::util::Or;
use crate::Map;

const THRESHOLD: usize = 30;

pub fn inline_lir<T: lir::Target>(
    lir: ir::Unit<Cfg<lir::Function<T>>>,
) -> lir::Unit<lir::Fallthrough> {
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

    let call_graph = CallGraph::new(&lir);
    let mut lir = lir.map(cfg::destruct_cfg);
    let mut inlined = 0;

    let main = symbol::intern_static(abi::XI_MAIN);
    let start = if lir.functions.contains_key(&main) {
        main
    } else if let Some(name) = lir.functions.keys().next() {
        *name
    } else {
        return lir;
    };

    for name in call_graph.postorder(&start) {
        let mut function = lir.functions.remove(&name).unwrap();

        function.statements = mem::take(&mut function.statements)
            .into_iter()
            .flat_map(|statement| match statement {
                lir::Statement::Call(
                    lir::Expression::Immediate(Immediate::Label(Label::Fixed(label))),
                    arguments,
                    returns,
                ) if lir.functions.contains_key(&label)
                    // Non-recursive
                    && !call_graph.is_recursive(&label)
                    && (
                        // Leaf function
                        call_graph.is_leaf(&label)

                        // Short function body
                        || lir.functions[&label].statements.len() < THRESHOLD

                        // Constant arguments
                        || arguments.iter().all(|expression| {
                            matches!(expression, lir::Expression::Immediate(_))
                    })
                    ) =>
                {
                    // Note: code duplication here is unfortunate, but if we move the
                    // conditions out of the match guard, we won't fall through and
                    // we'll have to reassemble the call if we decide not to inline.
                    log::trace!(
                        "Inlined callee {} ({}) into caller {}",
                        label,
                        if call_graph.is_leaf(&label) {
                            Cow::Borrowed("leaf function")
                        } else if lir.functions[&label].statements.len() < THRESHOLD {
                            Cow::Owned(format!(
                                "{} statements",
                                lir.functions[&label].statements.len()
                            ))
                        } else {
                            Cow::Borrowed("constant arguments")
                        },
                        function.name,
                    );
                    inlined += 1;

                    let statements = rewrite(&lir.functions[&label], arguments, returns);
                    Or::L(statements.into_iter())
                }
                statement => Or::R(iter::once(statement)),
            })
            .collect();

        lir.functions.insert(name, function);
    }

    log::debug!("Inlined {} call sites!", inlined);

    lir
}

fn rewrite(
    function: &lir::Function<lir::Fallthrough>,
    arguments: Vec<lir::Expression>,
    returns: Vec<Temporary>,
) -> Vec<lir::Statement<lir::Fallthrough>> {
    let mut rewriter = Rewriter {
        returns,
        rename_temporary: RefCell::default(),
        rename_label: RefCell::default(),
        rewritten: Vec::new(),
    };

    for (temporary, argument) in function.arguments.iter().copied().zip(arguments) {
        let temporary = rewriter.rewrite_temporary(&temporary);
        rewriter
            .rewritten
            .push(lir!((MOVE (TEMP temporary) argument)));
    }

    rewriter.rewrite_function(function);
    rewriter.rewritten
}

struct Rewriter {
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
                let returns = returns
                    .iter()
                    .map(|r#return| self.rewrite_temporary(r#return))
                    .collect();
                lir::Statement::Call(function, arguments, returns)
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
            // Note: assumes any labels used as expressions are globally visible, e.g.
            // function names, globals, static strings.
            lir::Expression::Immediate(immediate) => lir::Expression::Immediate(*immediate),
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
