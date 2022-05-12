use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::mem;

use crate::analyze::analyze;
use crate::analyze::UsedExpressions;
use crate::api::analyze::Analysis as _;
use crate::cfg::split_cfg;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::lir;

pub fn eliminate<T: lir::Target>(cfg: &mut Cfg<lir::Function<T>>) {
    split_cfg(cfg);

    let mut transformer = Transformer::new(cfg);

    for (label, statements) in cfg.blocks_mut() {
        transformer.eliminate_block(label, statements);
    }
}

struct Transformer<T> {
    latest: BTreeMap<Label, Vec<BTreeSet<lir::Expression>>>,
    used: BTreeMap<Label, Vec<BTreeSet<lir::Expression>>>,
    redundant: BTreeMap<lir::Expression, Temporary>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Transformer<T> {
    fn new(cfg: &Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze::<UsedExpressions<_>, _>(cfg);

        let mut used = BTreeMap::new();

        for (label, statements) in cfg.blocks() {
            let mut output = solution.inputs.remove(label).unwrap();
            let mut outputs = vec![output.clone()];

            for (index, statement) in statements.iter().enumerate().rev() {
                solution
                    .analysis
                    .transfer_with_metadata(label, index, statement, &mut output);
                outputs.push(output.clone());
            }

            outputs.reverse();
            used.insert(*label, outputs);
        }

        Self {
            latest: solution.analysis.latest,
            used,
            redundant: BTreeMap::new(),
            marker: PhantomData,
        }
    }

    fn eliminate_block(&mut self, label: &Label, statements: &mut Vec<lir::Statement<T>>) {
        for (index, statement) in mem::take(statements).into_iter().enumerate() {
            self.eliminate_statement(label, index, statement, statements);
        }
    }

    fn eliminate_statement(
        &mut self,
        label: &Label,
        index: usize,
        statement: lir::Statement<T>,
        statements: &mut Vec<lir::Statement<T>>,
    ) {
        match statement {
            statement @ (lir::Statement::Jump(_) | lir::Statement::Label(_)) => {
                statements.push(statement);
            }
            lir::Statement::CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => {
                let left = self.eliminate_expression(label, index, left, statements);
                let right = self.eliminate_expression(label, index, right, statements);
                statements.push(lir::Statement::CJump {
                    condition,
                    left,
                    right,
                    r#true,
                    r#false,
                });
            }
            lir::Statement::Call(function, arguments, returns) => {
                let function = self.eliminate_expression(label, index, function, statements);
                let arguments = arguments
                    .into_iter()
                    .map(|argument| self.eliminate_expression(label, index, argument, statements))
                    .collect();

                statements.push(lir::Statement::Call(function, arguments, returns));
            }
            lir::Statement::Move {
                destination,
                source,
            } => {
                let source = self.eliminate_expression(label, index, source, statements);
                statements.push(lir!((MOVE destination source)));
            }
            lir::Statement::Return(returns) => {
                let returns = returns
                    .into_iter()
                    .map(|r#return| self.eliminate_expression(label, index, r#return, statements))
                    .collect::<Vec<_>>();

                statements.push(lir::Statement::Return(returns));
            }
        }
    }

    fn eliminate_expression(
        &mut self,
        label: &Label,
        index: usize,
        expression: lir::Expression,
        statements: &mut Vec<lir::Statement<T>>,
    ) -> lir::Expression {
        match (
            self.latest[label][index].contains(&expression),
            self.used[label][index + 1].contains(&expression),
        ) {
            (true, true) => {
                let temporary = *self
                    .redundant
                    .entry(expression.clone())
                    .or_insert_with(|| Temporary::fresh("pre"));
                statements.push(lir!((MOVE (TEMP temporary) (expression))));
                lir::Expression::Temporary(temporary)
            }
            (true, false) => expression,
            (false, _) => match expression {
                expression @ (lir::Expression::Temporary(_)
                | lir::Expression::Immediate(_)
                | lir::Expression::Argument(_)
                | lir::Expression::Return(_)) => expression,
                expression @ (lir::Expression::Memory(_) | lir::Expression::Binary(_, _, _)) => {
                    let temporary = *self.redundant.get(&expression).unwrap();
                    lir::Expression::Temporary(temporary)
                }
            },
        }
    }
}
