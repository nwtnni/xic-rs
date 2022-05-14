use std::marker::PhantomData;
use std::mem;

use crate::analyze::analyze;
use crate::analyze::Analysis as _;
use crate::analyze::UsedExpressions;
use crate::cfg::split_cfg;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::lir;
use crate::Map;
use crate::Set;

pub fn eliminate_lir<T: lir::Target>(cfg: &mut Cfg<lir::Function<T>>) {
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
    latest: Map<Label, Vec<Set<lir::Expression>>>,
    used: Map<Label, Vec<Set<lir::Expression>>>,
    redundant: Map<lir::Expression, Temporary>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Transformer<T> {
    fn new(cfg: &Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze::<UsedExpressions<_>, _>(cfg);

        let mut used = Map::default();

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
            redundant: Map::default(),
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
        // Cache all live expressions created before this statement.
        //
        // The only two cases where we create an expression are:
        //
        // 1. Before a use site.
        //
        // Note that a use recursively uses all subexpressions,
        // so something like:
        //
        // ```text
        // (MOVE (TEMP a) (ADD (ADD (TEMP b) (TEMP c)) (TEMP d))
        // ```
        //
        // is conservatively assumed to use both subexpressions:
        //
        // ```text
        // (ADD (TEMP b) (TEMP c))
        // (ADD (ADD (TEMP b) (TEMP c)) (TEMP d))
        // ```
        //
        // This is because we can't statically determine which
        // of these computations will be useful to reuse later on,
        // so we keep both and eliminate dead code afterward.
        //
        // 2. Before a terminator.
        //
        // It's possible for an expression to be postponable at
        // the end of a basic block, but then be killed when merging
        // into a successor block. In this case, the latest point of
        // insertion is before the terminator, which doesn't necessarily
        // use the expression.
        //
        // ---
        //
        // For convenience and readability, we put this more general loop
        // here, but it would be possible to do more runtime verification
        // that these two cases hold if we checked against `statement` and
        // asserted that it was either a terminator or used a subexpression.
        for expression in self.latest[label][index]
            .iter()
            .filter(|expression| self.used[label][index + 1].contains(*expression))
            .cloned()
            .collect::<Vec<_>>()
        {
            let temporary = self.rewrite(expression.clone());
            statements.push(lir!((MOVE (TEMP temporary) expression)));
        }

        match statement {
            lir::Statement::Label(label) => {
                statements.push(lir::Statement::Label(label));
            }
            lir::Statement::Jump(target) => {
                statements.push(lir::Statement::Jump(target));
            }
            lir::Statement::CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => {
                let left = self.eliminate_expression(label, index, left);
                let right = self.eliminate_expression(label, index, right);
                statements.push(lir::Statement::CJump {
                    condition,
                    left,
                    right,
                    r#true,
                    r#false,
                });
            }
            lir::Statement::Call(function, arguments, returns) => {
                let function = self.eliminate_expression(label, index, function);
                let arguments = arguments
                    .into_iter()
                    .map(|argument| self.eliminate_expression(label, index, argument))
                    .collect();

                statements.push(lir::Statement::Call(function, arguments, returns));
            }
            lir::Statement::Move {
                destination,
                source,
            } => {
                let source = self.eliminate_expression(label, index, source);
                statements.push(lir!((MOVE destination source)));
            }
            lir::Statement::Return(returns) => {
                let returns = returns
                    .into_iter()
                    .map(|r#return| self.eliminate_expression(label, index, r#return))
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
    ) -> lir::Expression {
        // If no subexpressions were found to be redundant, then fall back to the original expression.
        self.eliminate_subexpression(label, index, expression.clone())
            .unwrap_or(expression)
    }

    fn eliminate_subexpression(
        &mut self,
        label: &Label,
        index: usize,
        expression: lir::Expression,
    ) -> Option<lir::Expression> {
        match (
            self.latest[label][index].contains(&expression),
            self.used[label][index + 1].contains(&expression),
        ) {
            // Expression was just defined, so rewrite.
            (true, true) => Some(lir::Expression::Temporary(self.rewrite(expression))),

            // Expression was defined, but is not used anywhere else. However, if we recurse,
            // we may be able to find a subexpression that has already been computed, and
            // rewrite this expression in terms of that.
            (true, false) => match expression {
                lir::Expression::Temporary(_)
                | lir::Expression::Immediate(_)
                | lir::Expression::Argument(_)
                | lir::Expression::Return(_) => None,
                lir::Expression::Memory(address) => {
                    let address = self.eliminate_subexpression(label, index, *address)?;
                    Some(lir::Expression::Memory(Box::new(address)))
                }
                lir::Expression::Binary(binary, left, right) => {
                    let (left, right) = match (
                        self.eliminate_subexpression(label, index, (&*left).clone())
                            .map(Box::new),
                        self.eliminate_subexpression(label, index, (&*right).clone())
                            .map(Box::new),
                    ) {
                        (None, None) => return None,
                        (None, Some(right)) => (left, right),
                        (Some(left), None) => (left, right),
                        (Some(left), Some(right)) => (left, right),
                    };

                    Some(lir::Expression::Binary(binary, left, right))
                }
            },

            // Expression must have been computed before, so rewrite.
            (false, _) => match expression {
                lir::Expression::Temporary(_)
                | lir::Expression::Immediate(_)
                | lir::Expression::Argument(_)
                | lir::Expression::Return(_) => None,
                expression @ (lir::Expression::Memory(_) | lir::Expression::Binary(_, _, _)) => {
                    Some(lir::Expression::Temporary(self.rewrite(expression)))
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
