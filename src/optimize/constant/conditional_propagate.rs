use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::ConditionalConstantPropagation;
use crate::analyze::Reachable;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;

pub fn conditional_propagate_lir(cfg: &mut Cfg<lir::Function<Label>>) {
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
                lir::Statement::Jump(_) => None,
                lir::Statement::CJump {
                    condition,
                    left,
                    right,
                    r#true,
                    r#false,
                } => match output.evaluate_condition(condition, left, right) {
                    None => {
                        propagate(left, &output);
                        propagate(right, &output);
                        None
                    }
                    Some(true) => {
                        cfg.graph.remove_edge(label, *r#false);
                        Some(lir::Statement::<Label>::Jump(*r#true))
                    }
                    Some(false) => {
                        cfg.graph.remove_edge(label, *r#true);
                        Some(lir::Statement::<Label>::Jump(*r#false))
                    }
                },
                lir::Statement::Call(function, arguments, _) => {
                    propagate(function, &output);
                    for argument in arguments {
                        propagate(argument, &output);
                    }
                    None
                }
                lir::Statement::Label(_) => None,
                lir::Statement::Move {
                    destination: _,
                    source,
                } => {
                    propagate(source, &output);
                    None
                }
                lir::Statement::Return(returns) => {
                    for r#return in returns {
                        propagate(r#return, &output);
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

// FIXME: propagate constants recursively
fn propagate(
    expression: &mut lir::Expression,
    output: &<ConditionalConstantPropagation as Analysis<lir::Function<Label>>>::Data,
) {
    if let Some(immediate) = output.evaluate_expression(expression) {
        *expression = lir::Expression::Immediate(immediate);
    }
}
