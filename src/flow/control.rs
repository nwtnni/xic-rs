use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::mem;

use petgraph::graphmap::DiGraphMap;

use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::sexp::Serialize;
use crate::data::symbol;

pub struct Control {
    name: symbol::Symbol,
    start: operand::Label,
    graph: DiGraphMap<operand::Label, Edge>,
    blocks: BTreeMap<operand::Label, Vec<lir::Statement<lir::Label>>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Edge {
    Unconditional,
    Conditional(bool),
}

pub fn construct_unit(unit: &ir::Unit<lir::Function<lir::Label>>) -> ir::Unit<Control> {
    unit.map(construct_function)
}

pub fn destruct_unit(unit: &ir::Unit<Control>) -> ir::Unit<lir::Function<lir::Fallthrough>> {
    unit.map(destruct_function)
}

enum State {
    Unreachable,
    Block(operand::Label, Vec<lir::Statement<lir::Label>>),
}

impl State {
    fn start(label: operand::Label) -> Self {
        State::Block(label, Vec::new())
    }

    fn push(&mut self, statement: lir::Statement<lir::Label>) {
        match self {
            State::Unreachable => (),
            State::Block(_, statements) => statements.push(statement),
        }
    }

    fn replace(
        &mut self,
        state: State,
    ) -> Option<(operand::Label, Vec<lir::Statement<lir::Label>>)> {
        match mem::replace(self, state) {
            State::Unreachable => None,
            State::Block(label, statements) => Some((label, statements)),
        }
    }
}

fn construct_function(function: &lir::Function<lir::Label>) -> Control {
    let mut graph = DiGraphMap::new();
    let mut blocks = BTreeMap::new();

    let start = operand::Label::fresh("start");

    let mut block = State::Block(start, Vec::new());

    for statement in &function.statements {
        match statement {
            lir::Statement::Jump(target) => {
                block.push(lir::Statement::Jump(*target));

                if let Some((label, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(label, *target, Edge::Unconditional);
                    blocks.insert(label, statements);
                }
            }
            lir::Statement::CJump(expression, r#true, r#false) => {
                block.push(lir::Statement::CJump(expression.clone(), *r#true, *r#false));

                if let Some((label, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(label, *r#true, Edge::Conditional(true));
                    graph.add_edge(label, r#false.0, Edge::Conditional(false));
                    blocks.insert(label, statements);
                }
            }
            lir::Statement::Label(next) => {
                if let Some((previous, mut statements)) = block.replace(State::start(*next)) {
                    statements.push(lir::Statement::Jump(*next));
                    graph.add_edge(previous, *next, Edge::Unconditional);
                    blocks.insert(previous, statements);
                }
            }
            lir::Statement::Return => {
                block.push(lir::Statement::Return);

                if let Some((previous, statements)) = block.replace(State::Unreachable) {
                    blocks.insert(previous, statements);
                }
            }
            lir::Statement::Call(function, arguments, returns) => block.push(lir::Statement::Call(
                function.clone(),
                arguments.clone(),
                *returns,
            )),
            lir::Statement::Move(into, from) => {
                block.push(lir::Statement::Move(into.clone(), from.clone()));
            }
        }
    }

    if let Some((label, statements)) = block.replace(State::Unreachable) {
        blocks.insert(label, statements);
    }

    Control {
        name: function.name,
        start,
        graph,
        blocks,
    }
}

fn destruct_function(function: &Control) -> lir::Function<lir::Fallthrough> {
    let mut dfs = vec![function.start];
    let mut statements = Vec::new();
    let mut visited = BTreeSet::new();

    while let Some(label) = dfs.pop() {
        if !visited.insert(label) {
            continue;
        }

        statements.push(lir::Statement::Label(label));
        statements.extend(function.blocks[&label].iter().map(fallthrough));

        let mut conditional = [None; 2];

        for (_, next, edge) in function.graph.edges(label) {
            match edge {
                Edge::Unconditional => dfs.push(next),
                Edge::Conditional(true) => conditional[0] = Some(next),
                Edge::Conditional(false) if !visited.contains(&next) => conditional[1] = Some(next),
                Edge::Conditional(false) => statements.push(lir::Statement::Jump(next)),
            }
        }

        dfs.extend(IntoIterator::into_iter(conditional).flatten());
    }

    lir::Function {
        name: function.name,
        statements,
    }
}

fn fallthrough(statement: &lir::Statement<lir::Label>) -> lir::Statement<lir::Fallthrough> {
    match statement {
        lir::Statement::Jump(label) => lir::Statement::Jump(*label),
        lir::Statement::CJump(condition, r#true, _) => {
            lir::Statement::CJump(condition.clone(), *r#true, lir::Fallthrough)
        }
        lir::Statement::Call(function, arguments, returns) => {
            lir::Statement::Call(function.clone(), arguments.clone(), *returns)
        }
        lir::Statement::Label(label) => lir::Statement::Label(*label),
        lir::Statement::Move(into, from) => lir::Statement::Move(into.clone(), from.clone()),
        lir::Statement::Return => lir::Statement::Return,
    }
}

impl fmt::Display for ir::Unit<Control> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "digraph {{")?;
        writeln!(fmt, "  label=\"{}\"", self.name)?;
        writeln!(fmt, "  node [shape=box nojustify=true]")?;

        for function in self.functions.values() {
            write!(fmt, "{}", function)?;
        }

        writeln!(fmt, "}}")
    }
}

impl fmt::Display for Control {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "  subgraph cluster_{} {{", self.name)?;
        writeln!(fmt, "    label=\"{}\"", self.name)?;

        for (label, statements) in &self.blocks {
            write!(fmt, "    \"{0}\" [label=\"\\\n{0}:\\l", label.sexp(),)?;

            for statement in statements {
                write!(
                    fmt,
                    "\\\n    {};\\l",
                    statement.sexp().to_string().replace('\n', "\\l\\\n    ")
                )?;
            }

            writeln!(fmt, "  \"];")?;
        }

        let mut edges = self.graph.all_edges().collect::<Vec<_>>();

        edges.sort();

        for (from, to, _) in edges {
            writeln!(fmt, r#"    "{}" -> "{}";"#, from.sexp(), to.sexp())?;
        }

        writeln!(fmt, "  }}")
    }
}
