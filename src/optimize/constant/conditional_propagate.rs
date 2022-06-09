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
                            propagate::<T>(left, &output);
                            propagate::<T>(right, &output);
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
                    propagate::<T>(function, &output);
                    for argument in arguments {
                        propagate::<T>(argument, &output);
                    }
                    None
                }
                lir::Statement::Move {
                    destination: lir::Expression::Temporary(_),
                    source,
                } => {
                    propagate::<T>(source, &output);
                    None
                }
                lir::Statement::Move {
                    destination: lir::Expression::Memory(address),
                    source,
                } => {
                    propagate::<T>(address, &output);
                    propagate::<T>(source, &output);
                    None
                }
                lir::Statement::Move { .. } => unreachable!(),
                lir::Statement::Return(returns) => {
                    for r#return in returns {
                        propagate::<T>(r#return, &output);
                    }
                    None
                }
            };

            solution.analysis.transfer(&transfer, &mut output);

            if let Some(rewrite) = rewrite {
                *statement = rewrite;
            }
        }
    }
}

fn propagate<T: lir::Target>(
    expression: &mut lir::Expression,
    output: &<ConditionalConstantPropagation as Analysis<lir::Function<T>>>::Data,
) {
    if let Some(Constant::Defined(immediate)) = output.evaluate_expression(expression) {
        *expression = lir::Expression::Immediate(immediate);
        return;
    }

    match expression {
        lir::Expression::Immediate(_) | lir::Expression::Temporary(_) => (),
        lir::Expression::Memory(address) => {
            propagate::<T>(address, output);
        }
        lir::Expression::Binary(_, left, right) => {
            propagate::<T>(left, output);
            propagate::<T>(right, output);
        }
    }
}
