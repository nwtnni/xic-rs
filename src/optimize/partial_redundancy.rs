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

/// Implements the lazy code motion algorithm for partial redundancy elimination,
/// from the research paper "Lazy Code Motion" by Knoop, Rüthing, and Steffen.
///
/// We use the following analyses and predicates:
///
/// ```text
/// anticipated
///     |      \
///     |   available
///     v      /
/// earliest  <
///     |   \
///     |  postponable
///     v    /
///  latest <
///     |  \
///     |  used
///     v   /
/// placed <
/// ```
///
/// - https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.92.4197&rep=rep1&type=pdf
/// - https://sites.cs.ucsb.edu/~yufeiding/cs293s/slides/293S_08_PRE_LCM.pdf
/// - http://www.cs.toronto.edu/~pekhimenko/courses/cscd70-w18/docs/Tutorial%205%20-%20Lazy%20Code%20Motion.pdf
/// - https://www.cs.utexas.edu/~pingali/CS380C/2020/lectures/LazyCodeMotion.pdf
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
            lir::Statement::Label(label) => {
                statements.push(lir::Statement::Label(label));
            }
            lir::Statement::Jump(target) => {
                for expression in self.latest[label][index]
                    .iter()
                    .filter(|expression| self.used[label][index + 1].contains(expression))
                    .cloned()
                    .collect::<Vec<_>>()
                {
                    let temporary = self.rewrite(expression.clone());
                    statements.push(lir!((MOVE (TEMP temporary) expression)));
                }

                statements.push(lir::Statement::Jump(target));
            }
            lir::Statement::CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => {
                for expression in self.latest[label][index]
                    .iter()
                    .filter(|expression| self.used[label][index + 1].contains(expression))
                    .cloned()
                    .collect::<Vec<_>>()
                {
                    let temporary = self.rewrite(expression.clone());
                    statements.push(lir!((MOVE (TEMP temporary) expression)));
                }

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
                let temporary = self.rewrite(expression.clone());
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
                    lir::Expression::Temporary(self.rewrite(expression))
                }
            },
        }
    }

    fn rewrite(&mut self, expression: lir::Expression) -> Temporary {
        *self
            .redundant
            .entry(expression)
            .or_insert_with(|| Temporary::fresh("pre"))
    }
}
