use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::analyze::PostponableExpressions;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;

pub struct UsedExpressions<T: lir::Target> {
    latest: BTreeMap<Label, Vec<BTreeSet<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for UsedExpressions<T> {
    const BACKWARD: bool = true;

    type Data = BTreeSet<lir::Expression>;

    fn new() -> Self {
        todo!()
    }

    fn new_with_metadata(cfg: &Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze::<PostponableExpressions<T>, lir::Function<T>>(cfg);

        let earliest = solution.analysis.earliest.clone();
        let mut postponable = BTreeMap::new();

        // Need random-ish access to `postponable` (or at least access to index + 1)
        for (label, statements) in cfg.blocks() {
            let mut output = solution.inputs.remove(label).unwrap();
            let mut outputs = vec![output.clone()];
            for (index, statement) in statements.iter().enumerate() {
                solution
                    .analysis
                    .transfer_with_metadata(label, index, statement, &mut output);
                outputs.push(output.clone());
            }
            postponable.insert(*label, outputs);
        }

        let mut latest = BTreeMap::new();

        for (label, statements) in cfg.blocks() {
            let mut outputs = Vec::new();

            for (index, statement) in statements.iter().enumerate() {
                let output = earliest[label][index]
                    .iter()
                    .chain(&postponable[label][index])
                    .filter(|expression| {
                        let used = match statement {
                            lir::Statement::Jump(_) | lir::Statement::Label(_) => false,
                            lir::Statement::CJump {
                                condition: _,
                                left,
                                right,
                                r#true: _,
                                r#false: _,
                            } => *expression == left || *expression == right,
                            lir::Statement::Call(function, arguments, _) => {
                                *expression == function || arguments.contains(expression)
                            }
                            lir::Statement::Move {
                                destination: _,
                                source,
                            } => *expression == source,
                            lir::Statement::Return(returns) => returns.contains(expression),
                        };

                        let last = !earliest[label][index + 1].contains(expression)
                            && !postponable[label][index + 1].contains(expression);

                        used || last
                    })
                    .cloned()
                    .collect::<BTreeSet<_>>();

                outputs.push(output);
            }

            let output = earliest[label][statements.len()]
                .iter()
                .chain(&postponable[label][statements.len()])
                .filter(|expression| {
                    cfg.outgoing(label).all(|successor| {
                        !earliest[&successor][0].contains(expression)
                            && !postponable[&successor][0].contains(expression)
                    })
                })
                .cloned()
                .collect::<BTreeSet<_>>();

            outputs.push(output);

            latest.insert(*label, outputs);
        }

        Self {
            latest,
            marker: PhantomData,
        }
    }

    fn default(&self) -> Self::Data {
        BTreeSet::new()
    }

    fn transfer(&self, _: &lir::Statement<T>, _: &mut Self::Data) {
        unreachable!()
    }

    fn transfer_with_metadata(
        &self,
        label: &Label,
        index: usize,
        statement: &lir::Statement<T>,
        output: &mut Self::Data,
    ) {
        match statement {
            lir::Statement::Jump(_) | lir::Statement::Label(_) => (),
            lir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                AnticipatedExpressions::insert(output, left);
                AnticipatedExpressions::insert(output, right);
            }
            lir::Statement::Call(function, arguments, _) => {
                AnticipatedExpressions::insert(output, function);
                for argument in arguments {
                    AnticipatedExpressions::insert(output, argument);
                }
            }
            lir::Statement::Move {
                destination: _,
                source,
            } => {
                AnticipatedExpressions::insert(output, source);
            }
            lir::Statement::Return(returns) => {
                for r#return in returns {
                    AnticipatedExpressions::insert(output, r#return);
                }
            }
        }

        output.retain(|expression| !self.latest[label][index].contains(expression));
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        input.clear();
        input.extend(outputs.flatten().flatten().cloned());
    }
}
