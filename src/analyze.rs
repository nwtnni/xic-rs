mod anticipated_expressions;
mod available_expressions;
mod call_graph;
mod constant_propagation;
mod copy_propagation;
mod dot;
mod live_ranges;
mod live_variables;
mod postponable_expressions;

pub use anticipated_expressions::AnticipatedExpressions;
pub use available_expressions::AvailableExpressions;
pub use call_graph::CallGraph;
pub use constant_propagation::ConstantPropagation;
pub use copy_propagation::CopyPropagation;
pub use dot::display;
pub use live_ranges::LiveRanges;
pub use live_ranges::Range;
pub use live_variables::LiveVariables;
pub use postponable_expressions::PostponableExpressions;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use petgraph::Direction;

use crate::cfg::Cfg;
use crate::cfg::Function;
use crate::data::operand::Label;

pub trait Analysis<T: Function>: Sized {
    const BACKWARD: bool;

    type Data: Clone + Eq;

    fn new() -> Self;

    fn new_with_metadata(_cfg: &Cfg<T>) -> Self {
        Self::new()
    }

    fn default(&self) -> Self::Data;

    fn default_with_metadata(&self, _label: &Label) -> Self::Data {
        self.default()
    }

    fn transfer(&self, statement: &T::Statement, output: &mut Self::Data);

    fn transfer_with_metadata(
        &self,
        _label: &Label,
        _index: usize,
        statement: &T::Statement,
        output: &mut Self::Data,
    ) {
        self.transfer(statement, output);
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a;

    fn merge_with_metadata<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = (Label, Option<&'a Self::Data>)>,
        Self::Data: 'a,
    {
        self.merge(outputs.map(|(_, data)| data), input)
    }
}

pub struct Solution<A: Analysis<T>, T: Function> {
    pub analysis: A,
    pub inputs: BTreeMap<Label, A::Data>,
    pub outputs: BTreeMap<Label, A::Data>,
}

pub fn analyze<A: Analysis<T>, T: Function>(cfg: &Cfg<T>) -> Solution<A, T> {
    let analysis = A::new_with_metadata(cfg);

    let strongly_connected_components = cfg.strongly_connected_components(A::BACKWARD);
    let (predecessors, successors) = match A::BACKWARD {
        true => (Direction::Outgoing, Direction::Incoming),
        false => (Direction::Incoming, Direction::Outgoing),
    };

    let mut inputs = BTreeMap::<Label, A::Data>::new();
    let mut outputs = BTreeMap::<Label, A::Data>::new();
    let mut worklist = VecDeque::new();
    let mut workset = BTreeSet::new();

    // Topological order
    for component in strongly_connected_components.into_iter().rev() {
        // Reverse postorder
        worklist.extend(component.iter().copied().rev());
        workset.extend(component.iter().copied().rev());

        let component = component.into_iter().collect::<BTreeSet<_>>();

        while let Some(label) = worklist.pop_front() {
            workset.remove(&label);

            let input = inputs
                .entry(label)
                .or_insert_with(|| analysis.default_with_metadata(&label));

            analysis.merge_with_metadata(
                cfg.neighbors(predecessors, &label)
                    .map(|predecessor| (predecessor, outputs.get(&predecessor))),
                input,
            );

            let mut output = input.clone();

            if A::BACKWARD {
                for (index, statement) in cfg[&label].iter().enumerate().rev() {
                    analysis.transfer_with_metadata(&label, index, statement, &mut output);
                }
            } else {
                for (index, statement) in cfg[&label].iter().enumerate() {
                    analysis.transfer_with_metadata(&label, index, statement, &mut output);
                }
            }

            match outputs.get_mut(&label) {
                Some(existing) if *existing == output => continue,
                Some(existing) => *existing = output,
                None => {
                    outputs.insert(label, output);
                }
            };

            cfg.neighbors(successors, &label)
                .filter(|successor| component.contains(successor))
                .filter(|successor| workset.insert(*successor))
                .for_each(|successor| worklist.push_back(successor));
        }
    }

    Solution {
        analysis,
        inputs,
        outputs,
    }
}
