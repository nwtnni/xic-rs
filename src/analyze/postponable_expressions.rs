use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::analyze::AvailableExpressions;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;

pub struct PostponableExpressions<T: lir::Target> {
    earliest: BTreeMap<Label, Vec<BTreeSet<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for PostponableExpressions<T> {
    const BACKWARD: bool = false;

    type Data = BTreeSet<lir::Expression>;

    fn new() -> Self {
        unreachable!()
    }

    fn new_with_metadata(cfg: &Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze::<AvailableExpressions<T>, lir::Function<T>>(cfg);
        let mut earliest = BTreeMap::new();

        for (label, statements) in cfg.blocks() {
            let mut anticipated_output =
                solution.analysis.anticipated.inputs.remove(label).unwrap();

            let mut outputs = vec![anticipated_output.clone()];

            for (index, statement) in statements.iter().enumerate().rev() {
                solution
                    .analysis
                    .anticipated
                    .analysis
                    .transfer_with_metadata(label, index, statement, &mut anticipated_output);
                outputs.push(anticipated_output.clone());
            }

            outputs.reverse();

            let mut available_output = solution.inputs.remove(label).unwrap();

            outputs[0].retain(|expression| !available_output.contains(expression));

            for (index, statement) in statements.iter().enumerate() {
                solution.analysis.transfer_with_metadata(
                    label,
                    index,
                    statement,
                    &mut available_output,
                );
                outputs[index + 1].retain(|expression| !available_output.contains(expression));
            }

            earliest.insert(*label, outputs);
        }

        Self {
            earliest,
            marker: PhantomData,
        }
    }

    fn default(&self) -> Self::Data {
        unreachable!()
    }

    fn default_with_metadata(&self, label: &Label) -> Self::Data {
        self.earliest[label][0].clone()
    }

    fn transfer(&self, statement: &lir::Statement<T>, output: &mut Self::Data) {
        match statement {
            lir::Statement::Jump(_) | lir::Statement::Label(_) => (),
            lir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                AnticipatedExpressions::remove(output, left);
                AnticipatedExpressions::remove(output, right);
            }
            lir::Statement::Call(function, arguments, _) => {
                AnticipatedExpressions::remove(output, function);
                for argument in arguments {
                    AnticipatedExpressions::remove(output, argument);
                }
            }
            lir::Statement::Move {
                destination: _,
                source,
            } => {
                AnticipatedExpressions::remove(output, source);
            }
            lir::Statement::Return(returns) => {
                for r#return in returns {
                    AnticipatedExpressions::remove(output, r#return);
                }
            }
        }
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        <AnticipatedExpressions as Analysis<lir::Function<T>>>::merge(
            &AnticipatedExpressions,
            outputs,
            input,
        )
    }
}
