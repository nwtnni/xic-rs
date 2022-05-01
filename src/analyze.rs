#![allow(dead_code)]

use std::collections::BTreeMap;
use std::collections::VecDeque;

use petgraph::graphmap::NeighborsDirected;

use crate::cfg::Cfg;
use crate::cfg::Function;
use crate::data::operand::Label;

pub trait Analysis<T: Function> {
    type Data;
    type Direction: Direction<T>;

    fn initialize(&mut self, _cfg: &Cfg<T>) {}

    fn default(&mut self, cfg: &Cfg<T>, label: &Label) -> Self::Data;

    fn transfer_statement(
        &mut self,
        statement: &T::Statement,
        input: &Self::Data,
        output: &mut Self::Data,
    ) -> bool;

    fn transfer_statements(
        &mut self,
        statements: &[T::Statement],
        input: &Self::Data,
        output: &mut Self::Data,
    ) -> bool {
        let mut changed = false;
        for statement in statements {
            changed |= self.transfer_statement(statement, input, output);
        }
        changed
    }

    fn merge(&mut self, output: &Self::Data, input: &mut Self::Data);
}

pub trait Direction<T: Function> {
    fn worklist(cfg: &Cfg<T>) -> VecDeque<Label>;
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
    fn worklist(cfg: &Cfg<T>) -> VecDeque<Label> {
        let mut worklist = VecDeque::new();
        worklist.push_back(*cfg.exit());
        worklist
    }

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
    fn worklist(cfg: &Cfg<T>) -> VecDeque<Label> {
        let mut worklist = VecDeque::new();
        worklist.push_back(*cfg.enter());
        worklist
    }

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

pub fn analyze<A: Analysis<T>, T: Function>(analysis: &mut A, cfg: &Cfg<T>) -> Solution<A, T> {
    analysis.initialize(cfg);

    let mut worklist = A::Direction::worklist(cfg);
    let mut inputs = BTreeMap::<Label, A::Data>::new();
    let mut outputs = BTreeMap::<Label, A::Data>::new();

    while let Some(label) = worklist.pop_front() {
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

        if analysis.transfer_statements(&cfg[&label], input, output) {
            worklist.extend(A::Direction::successors(cfg, &label));
        }
    }

    Solution { inputs, outputs }
}
