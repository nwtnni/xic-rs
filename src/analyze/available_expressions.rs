use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::data::lir;
use crate::data::operand::Label;

pub struct AvailableExpressions<T: lir::Target> {
    pub(super) anticipated: BTreeMap<Label, Vec<BTreeSet<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for AvailableExpressions<T> {
    const BACKWARD: bool = false;

    type Data = BTreeSet<lir::Expression>;

    fn new() -> Self {
        unreachable!()
    }

    fn new_with_metadata(cfg: &crate::cfg::Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze::<AnticipatedExpressions, _>(cfg);
        let mut anticipated = BTreeMap::new();

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
            anticipated.insert(*label, outputs);
        }

        Self {
            anticipated,
            marker: PhantomData,
        }
    }

    fn default(&self) -> Self::Data {
        unreachable!()
    }

    fn default_with_metadata(&self, label: &Label) -> Self::Data {
        self.anticipated[label][0].clone()
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
        output.extend(self.anticipated[label][index].iter().cloned());

        match statement {
            lir::Statement::Jump(_)
            | lir::Statement::CJump { .. }
            | lir::Statement::Label(_)
            | lir::Statement::Return(_) => (),
            lir::Statement::Call(_, _, returns) => {
                for r#return in 0..*returns {
                    AnticipatedExpressions::remove(output, &lir::Expression::Return(r#return));
                }
            }
            lir::Statement::Move {
                destination,
                source: _,
            } => {
                AnticipatedExpressions::remove(output, destination);
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
        );
    }
}
