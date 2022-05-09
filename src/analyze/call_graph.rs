use petgraph::graphmap::DiGraphMap;
use petgraph::visit::Walker as _;
use petgraph::Direction;

use crate::data::lir;
use crate::data::operand::{Immediate, Label};
use crate::data::symbol::Symbol;

pub struct CallGraph(DiGraphMap<Symbol, ()>);

impl CallGraph {
    pub fn new<T: lir::Target>(lir: &lir::Unit<T>) -> Self {
        let mut graph = lir
            .functions
            .iter()
            .flat_map(|(caller, function)| {
                function
                    .statements
                    .iter()
                    .map(move |statement| (caller, statement))
            })
            .filter_map(|(caller, statement)| match statement {
                lir::Statement::Call(
                    lir::Expression::Immediate(Immediate::Label(Label::Fixed(callee))),
                    _,
                    _,
                ) => Some((*caller, *callee)),
                _ => None,
            })
            .collect::<DiGraphMap<_, _>>();

        for name in lir.functions.keys() {
            graph.add_node(*name);
        }

        Self(graph)
    }

    pub fn is_recursive(&self, name: &Symbol) -> bool {
        self.0.contains_edge(*name, *name)
    }

    pub fn is_leaf(&self, name: &Symbol) -> bool {
        self.0
            .neighbors_directed(*name, Direction::Outgoing)
            .count()
            == 0
    }

    pub fn postorder(&self, start: &Symbol) -> impl Iterator<Item = Symbol> + '_ {
        petgraph::visit::DfsPostOrder::new(&self.0, *start).iter(&self.0)
    }
}
