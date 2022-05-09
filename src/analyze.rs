mod call_graph;
mod constant_propagation;
mod copy_propagation;
mod dot;
mod live_ranges;
mod live_variables;

pub use call_graph::CallGraph;
pub use constant_propagation::ConstantPropagation;
pub use copy_propagation::CopyPropagation;
pub use dot::display;
pub use live_ranges::LiveRanges;
pub use live_ranges::Range;
pub use live_variables::LiveVariables;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use petgraph::graphmap::NeighborsDirected;

use crate::cfg::Cfg;
use crate::cfg::Function;
use crate::data::operand::Label;

pub trait Analysis<T: Function>: Sized {
    type Data: Clone + Eq;
    type Direction: Direction<T>;

    fn new(cfg: &Cfg<T>) -> Self;

    fn default(&self) -> Self::Data;

    fn transfer(&self, statement: &T::Statement, output: &mut Self::Data);

    fn merge<'a, I>(&'a self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = &'a Self::Data>,
        Self::Data: 'a;
}

pub trait Direction<T: Function> {
    const REVERSE: bool;
    fn successors<'cfg>(
        cfg: &'cfg Cfg<T>,
        label: &Label,
    ) -> NeighborsDirected<'cfg, Label, petgraph::Directed>;
    fn predecessors<'cfg>(
        cfg: &'cfg Cfg<T>,
        label: &Label,
    ) -> NeighborsDirected<'cfg, Label, petgraph::Directed>;
}

pub struct Backward;

impl<T: Function> Direction<T> for Backward {
    const REVERSE: bool = true;

    fn successors<'cfg>(
        cfg: &'cfg Cfg<T>,
        label: &Label,
    ) -> NeighborsDirected<'cfg, Label, petgraph::Directed> {
        cfg.incoming(label)
    }

    fn predecessors<'cfg>(
        cfg: &'cfg Cfg<T>,
        label: &Label,
    ) -> NeighborsDirected<'cfg, Label, petgraph::Directed> {
        cfg.outgoing(label)
    }
}

pub struct Forward;

impl<T: Function> Direction<T> for Forward {
    const REVERSE: bool = false;

    fn successors<'cfg>(
        cfg: &'cfg Cfg<T>,
        label: &Label,
    ) -> NeighborsDirected<'cfg, Label, petgraph::Directed> {
        cfg.outgoing(label)
    }

    fn predecessors<'cfg>(
        cfg: &'cfg Cfg<T>,
        label: &Label,
    ) -> NeighborsDirected<'cfg, Label, petgraph::Directed> {
        cfg.incoming(label)
    }
}

pub struct Solution<A: Analysis<T>, T: Function> {
    pub analysis: A,
    pub inputs: BTreeMap<Label, A::Data>,
    pub outputs: BTreeMap<Label, A::Data>,
}

pub fn analyze<A: Analysis<T>, T: Function>(cfg: &Cfg<T>) -> Solution<A, T> {
    let analysis = A::new(cfg);

    let strongly_connected_components = cfg.strongly_connected_components(A::Direction::REVERSE);

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

            let input = inputs.entry(label).or_insert_with(|| analysis.default());

            for predecessor in A::Direction::predecessors(cfg, &label) {
                outputs
                    .entry(predecessor)
                    .or_insert_with(|| analysis.default());
            }

            analysis.merge(
                A::Direction::predecessors(cfg, &label).map(|predecessor| &outputs[&predecessor]),
                input,
            );

            let mut output = input.clone();

            if A::Direction::REVERSE {
                for statement in cfg[&label].iter().rev() {
                    analysis.transfer(statement, &mut output);
                }
            } else {
                for statement in &cfg[&label] {
                    analysis.transfer(statement, &mut output);
                }
            }

            match outputs.get_mut(&label) {
                Some(existing) if *existing == output => continue,
                Some(existing) => *existing = output,
                None => {
                    outputs.insert(label, output);
                }
            };

            A::Direction::successors(cfg, &label)
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
