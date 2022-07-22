use std::marker::PhantomData;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::AvailableExpressions;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;
use crate::Map;
use crate::Set;

pub struct Earliest<T> {
    inner: Map<Label, Vec<Set<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Earliest<T> {
    pub fn new(cfg: &Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze(AvailableExpressions::new(cfg), cfg);

        for (label, statements) in cfg.blocks() {
            let mut output = solution.inputs.remove(label).unwrap();

            solution.analysis.anticipated.get_mut(label).unwrap()[0]
                .retain(|expression| !output.contains(expression));

            for (index, statement) in statements.iter().enumerate() {
                solution
                    .analysis
                    .transfer_with_metadata(label, index, statement, &mut output);
                solution.analysis.anticipated.get_mut(label).unwrap()[index + 1]
                    .retain(|expression| !output.contains(expression));
            }
        }

        Self {
            inner: solution.analysis.anticipated,
            marker: PhantomData,
        }
    }

    pub(super) fn into_inner(self) -> Map<Label, Vec<Set<lir::Expression>>> {
        self.inner
    }
}

impl<T: lir::Target> Analysis<lir::Function<T>> for Earliest<T> {
    const BACKWARD: bool = false;

    type Data = Set<lir::Expression>;

    fn default(&self) -> Self::Data {
        unreachable!()
    }

    fn default_with_metadata(&self, label: &Label) -> Self::Data {
        self.inner[label][0].clone()
    }

    fn transfer(&self, _: &lir::Statement<T>, _: &mut Self::Data) {
        unreachable!()
    }

    fn transfer_with_metadata(
        &self,
        label: &Label,
        index: usize,
        _: &lir::Statement<T>,
        output: &mut Self::Data,
    ) {
        output.clear();
        output.extend(self.inner[label][index + 1].iter().cloned());
    }

    fn merge<'a, I>(&self, _: I, _: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
    }
}
