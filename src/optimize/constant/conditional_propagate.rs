use petgraph::Direction;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::ConditionalConstantPropagation;
use crate::analyze::Constant;
use crate::analyze::Reachable;
use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::data::lir;
use crate::util;

pub fn conditional_propagate_lir<T: lir::Target>(cfg: &mut Cfg<lir::Function<T>>) {
    log::info!(
        "[{}] Conditionally propagating constants in {}...",
        std::any::type_name::<Cfg<lir::Function<T>>>(),
        cfg.name(),
    );
    util::time!(
        "[{}] Done conditionally propagating constants in {}",
        std::any::type_name::<Cfg<lir::Function<T>>>(),
        cfg.name(),
    );

    let mut solution = analyze::<ConditionalConstantPropagation, _>(cfg);
    let mut propagated = 0;
    let mut rewritten = 0;

    for label in cfg.blocks.keys().copied().collect::<Vec<_>>() {
        let mut output = solution.inputs.remove(&label).unwrap();

        match output.reachable {
            Reachable::Branch(_, _) => unreachable!(),
            Reachable::Linear(true) => (),
            Reachable::Linear(false) => {
                cfg.graph.remove_node(label);
                cfg.blocks.remove(&label);
                continue;
            }
        }

        let statements = cfg.blocks.get_mut(&label).unwrap();

        for statement in statements {
            let transfer = statement.clone();

            let rewrite = match statement {
                lir::Statement::Jump(_) | lir::Statement::Label(_) => None,
                lir::Statement::CJump {
                    condition,
                    left,
                    right,
                    r#true,
                    r#false,
                } => {
                    let r#false = r#false.target().copied().or_else(|| {
                        cfg.graph
                            .edges_directed(label, Direction::Outgoing)
                            .find_map(|(_, successor, edge)| match edge {
                                Edge::Conditional(false) => Some(successor),
                                Edge::Unconditional | Edge::Conditional(true) => None,
                            })
                    });

                    match output.evaluate_condition(condition, left, right) {
                        None => {
                            propagate::<T>(&output, &mut propagated, left);
                            propagate::<T>(&output, &mut propagated, right);
                            None
                        }
                        Some(true) => {
                            // Note: may already be removed if unreachable.
                            if let Some(r#false) = r#false {
                                cfg.graph.remove_edge(label, r#false);
                            }
                            cfg.graph
                                .add_edge(label, *r#true, Edge::Unconditional)
                                .unwrap();
                            Some(lir::Statement::<T>::Jump(*r#true))
                        }
                        Some(false) => {
                            let r#false = r#false.unwrap();
                            cfg.graph.remove_edge(label, *r#true);
                            cfg.graph
                                .add_edge(label, r#false, Edge::Unconditional)
                                .unwrap();
                            Some(lir::Statement::<T>::Jump(r#false))
                        }
                    }
                }
                lir::Statement::Call(function, arguments, _) => {
                    propagate::<T>(&output, &mut propagated, function);
                    for argument in arguments {
                        propagate::<T>(&output, &mut propagated, argument);
                    }
                    None
                }
                lir::Statement::Move {
                    destination: lir::Expression::Temporary(_),
                    source,
                } => {
                    propagate::<T>(&output, &mut propagated, source);
                    None
                }
                lir::Statement::Move {
                    destination: lir::Expression::Memory(address),
                    source,
                } => {
                    propagate::<T>(&output, &mut propagated, address);
                    propagate::<T>(&output, &mut propagated, source);
                    None
                }
                lir::Statement::Move { .. } => unreachable!(),
                lir::Statement::Return(returns) => {
                    for r#return in returns {
                        propagate::<T>(&output, &mut propagated, r#return);
                    }
                    None
                }
            };

            solution.analysis.transfer(&transfer, &mut output);

            if let Some(rewrite) = rewrite {
                log::trace!("Rewrote {} into {}", statement, rewrite);
                rewritten += 1;
                *statement = rewrite;
            }
        }
    }

    log::debug!(
        "Propagated {} LIR constants and rewrote {} LIR branches!",
        propagated,
        rewritten,
    );
}

fn propagate<T: lir::Target>(
    output: &<ConditionalConstantPropagation as Analysis<lir::Function<T>>>::Data,
    propagated: &mut usize,
    expression: &mut lir::Expression,
) {
    if let Some(Constant::Defined(immediate)) = output.evaluate_expression(expression) {
        if !matches!(expression, lir::Expression::Immediate(expression) if *expression == immediate)
        {
            log::trace!("Replaced {} with {}", expression, immediate);
            *propagated += 1;
            *expression = lir::Expression::Immediate(immediate);
            return;
        }
    }

    match expression {
        lir::Expression::Immediate(_) | lir::Expression::Temporary(_) => (),
        lir::Expression::Memory(address) => {
            propagate::<T>(output, propagated, address);
        }
        lir::Expression::Binary(_, left, right) => {
            propagate::<T>(output, propagated, left);
            propagate::<T>(output, propagated, right);
        }
    }
}
