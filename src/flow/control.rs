use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::mem;

use petgraph::graphmap::DiGraphMap;

use crate::abi;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Label;
use crate::data::sexp::Serialize;
use crate::data::symbol::Symbol;

pub struct Control {
    name: Symbol,
    enter: Label,
    #[allow(dead_code)]
    exit: Label,
    graph: DiGraphMap<Label, Edge>,
    blocks: BTreeMap<Label, Vec<lir::Statement<lir::Label>>>,
    arguments: usize,
    returns: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Edge {
    Unconditional,
    Conditional(bool),
}

pub fn construct_unit(
    unit: &ir::Unit<ir::Function<Vec<lir::Statement<lir::Label>>>>,
) -> ir::Unit<Control> {
    unit.map(construct_function)
}

pub fn destruct_unit(
    unit: &ir::Unit<Control>,
) -> ir::Unit<ir::Function<Vec<lir::Statement<lir::Fallthrough>>>> {
    unit.map(destruct_function)
}

enum State {
    Unreachable,
    Block(Label, Vec<lir::Statement<lir::Label>>),
}

impl State {
    fn block(label: Label) -> Self {
        State::Block(label, Vec::new())
    }

    fn push(&mut self, statement: lir::Statement<lir::Label>) {
        match self {
            State::Unreachable => (),
            State::Block(_, statements) => statements.push(statement),
        }
    }

    fn replace(&mut self, state: State) -> Option<(Label, Vec<lir::Statement<lir::Label>>)> {
        match mem::replace(self, state) {
            State::Unreachable => None,
            State::Block(label, statements) => Some((label, statements)),
        }
    }
}

fn construct_function(function: &ir::Function<Vec<lir::Statement<lir::Label>>>) -> Control {
    let mut graph = DiGraphMap::new();
    let mut blocks = BTreeMap::new();

    let enter = Label::fresh("enter");
    let exit = Label::fresh("exit");

    blocks.insert(exit, Vec::new());

    let mut block = State::Block(enter, Vec::new());

    for statement in &function.statements {
        match statement {
            jump @ lir::Statement::Jump(target) => {
                block.push(jump.clone());

                if let Some((label, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(label, *target, Edge::Unconditional);
                    blocks.insert(label, statements);
                }
            }
            cjump @ lir::Statement::CJump {
                condition: _,
                left: _,
                right: _,
                r#true,
                r#false,
            } => {
                block.push(cjump.clone());

                if let Some((label, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(label, *r#true, Edge::Conditional(true));
                    graph.add_edge(label, r#false.0, Edge::Conditional(false));
                    blocks.insert(label, statements);
                }
            }
            lir::Statement::Label(next) => {
                if let Some((previous, mut statements)) = block.replace(State::block(*next)) {
                    statements.push(lir::Statement::Jump(*next));
                    graph.add_edge(previous, *next, Edge::Unconditional);
                    blocks.insert(previous, statements);
                }
            }
            r#return @ lir::Statement::Return(_) => {
                block.push(r#return.clone());

                // Insert edge to dummy exit node for dataflow analysis
                if let Some((previous, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(previous, exit, Edge::Unconditional);
                    blocks.insert(previous, statements);
                }
            }
            // Special-case ABI function that never returns
            call @ lir::Statement::Call(function, _, _)
                if function.is_label(abi::XI_OUT_OF_BOUNDS) =>
            {
                block.push(call.clone());

                // Insert edge to dummy exit node for dataflow analysis
                if let Some((previous, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(previous, exit, Edge::Unconditional);
                    blocks.insert(previous, statements);
                }
            }

            call @ lir::Statement::Call(_, _, _) => block.push(call.clone()),
            r#move @ lir::Statement::Move { .. } => block.push(r#move.clone()),
        }
    }

    if let Some((label, statements)) = block.replace(State::Unreachable) {
        blocks.insert(label, statements);
    }

    Control {
        name: function.name,
        enter,
        exit,
        graph,
        blocks,
        arguments: function.arguments,
        returns: function.returns,
    }
}

fn destruct_function(function: &Control) -> lir::Function<lir::Fallthrough> {
    let mut dfs = vec![function.enter];
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

    ir::Function {
        name: function.name,
        statements,
        arguments: function.arguments,
        returns: function.returns,
    }
}

fn fallthrough(statement: &lir::Statement<lir::Label>) -> lir::Statement<lir::Fallthrough> {
    match statement {
        lir::Statement::Jump(label) => lir::Statement::Jump(*label),
        lir::Statement::CJump {
            condition,
            left,
            right,
            r#true,
            r#false: _,
        } => lir::Statement::CJump {
            condition: *condition,
            left: left.clone(),
            right: right.clone(),
            r#true: *r#true,
            r#false: lir::Fallthrough,
        },
        lir::Statement::Call(function, arguments, returns) => {
            lir::Statement::Call(function.clone(), arguments.clone(), *returns)
        }
        lir::Statement::Label(label) => lir::Statement::Label(*label),
        lir::Statement::Move {
            destination,
            source,
        } => lir::Statement::Move {
            destination: destination.clone(),
            source: source.clone(),
        },
        lir::Statement::Return(returns) => lir::Statement::Return(returns.clone()),
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
            write!(fmt, "    \"{0}\" [label=\"\\\n{0}:\\l", label)?;

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
            writeln!(fmt, r#"    "{}" -> "{}";"#, from, to)?;
        }

        writeln!(fmt, "  }}")
    }
}
