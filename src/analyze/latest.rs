use std::marker::PhantomData;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::PostponableExpressions;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;
use crate::Map;
use crate::Set;

pub struct Latest<T> {
    inner: Map<Label, Vec<Set<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Latest<T> {
    pub fn new(cfg: &Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze(PostponableExpressions::new(cfg), cfg);
        let mut postponable = Map::default();

        // Need random access to `postponable` (or at least access to
        // index + 1, and index 0 of successors)
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

        for (label, statements) in cfg.blocks() {
            for (index, _) in statements
                .iter()
                .enumerate()
                .take(statements.len().saturating_sub(1))
            {
                let latest = solution
                    .analysis
                    .earliest
                    .get_mut(label)
                    .unwrap()
                    .get_mut(index)
                    .unwrap();

                latest.extend(postponable[label][index].iter().cloned());
                latest.retain(|expression| !postponable[label][index + 1].contains(expression));
            }

            // Special case: if our basic blocks end with a `CJUMP` or `JUMP`,
            // we shouldn't insert any code after those instructions. So the `latest`
            // predicate should forward any latest expressions from after the terminator
            // to before.
            match statements.last() {
                None => continue,
                Some(lir::Statement::Jump(_) | lir::Statement::CJump { .. }) => (),
                Some(_) => unreachable!(),
            }

            // Nothing is latest after terminator
            solution
                .analysis
                .earliest
                .get_mut(label)
                .unwrap()
                .get_mut(statements.len())
                .unwrap()
                .clear();

            let latest = solution
                .analysis
                .earliest
                .get_mut(label)
                .unwrap()
                .get_mut(statements.len() - 1)
                .unwrap();

            latest.extend(postponable[label][statements.len() - 1].iter().cloned());
            latest.extend(postponable[label][statements.len()].iter().cloned());
            latest.retain(|expression| {
                !postponable[label][statements.len()].contains(expression)
                    || cfg
                        .outgoing(label)
                        .any(|successor| !postponable[&successor][0].contains(expression))
            });
        }

        Self {
            inner: solution.analysis.earliest,
            marker: PhantomData,
        }
    }

    pub(super) fn into_inner(self) -> Map<Label, Vec<Set<lir::Expression>>> {
        self.inner
    }
}

impl<T: lir::Target> Analysis<lir::Function<T>> for Latest<T> {
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
