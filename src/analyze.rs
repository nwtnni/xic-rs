mod dot;
mod live_ranges;
mod live_variables;

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
    type Data: Clone;
    type Direction: Direction<T>;

    fn new(cfg: &Cfg<T>) -> Self;

    fn default(&self, cfg: &Cfg<T>, label: &Label) -> Self::Data;

    fn transfer(&self, statements: &T::Statement, output: &mut Self::Data) -> bool;

    fn merge(&self, output: &Self::Data, input: &mut Self::Data);
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
    pub inputs: BTreeMap<Label, A::Data>,
    pub outputs: BTreeMap<Label, A::Data>,
}

pub fn analyze<A: Analysis<T>, T: Function>(cfg: &Cfg<T>) -> (A, Solution<A, T>) {
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

            let input = inputs
                .entry(label)
                .or_insert_with(|| analysis.default(cfg, &label));

            for predecessor in A::Direction::predecessors(cfg, &label) {
                let output = outputs
                    .entry(predecessor)
                    .or_insert_with(|| analysis.default(cfg, &label));

                analysis.merge(output, input);
            }

            let output = outputs
                .entry(label)
                .or_insert_with(|| analysis.default(cfg, &label));

            *output = input.clone();
            let mut changed = false;

            if A::Direction::REVERSE {
                for statement in cfg[&label].iter().rev() {
                    changed |= analysis.transfer(statement, output);
                }
            } else {
                for statement in &cfg[&label] {
                    changed |= analysis.transfer(statement, output);
                }
            }

            if !changed {
                continue;
            }

            A::Direction::successors(cfg, &label)
                .filter(|successor| component.contains(successor))
                .filter(|successor| workset.insert(*successor))
                .for_each(|successor| worklist.push_back(successor));
        }
    }

    (analysis, Solution { inputs, outputs })
}
