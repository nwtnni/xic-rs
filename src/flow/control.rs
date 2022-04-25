use std::collections::BTreeMap;
use std::mem;

use petgraph::graphmap::DiGraphMap;

use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::symbol;

#[allow(dead_code)]
pub struct Control {
    name: symbol::Symbol,
    start: operand::Label,
    graph: DiGraphMap<operand::Label, ()>,
    blocks: BTreeMap<operand::Label, Vec<lir::Statement>>,
}

pub fn construct_unit(unit: &ir::Unit<lir::Function>) -> ir::Unit<Control> {
    unit.map(construct_function)
}

enum State {
    Unreachable,
    Block(operand::Label, Vec<lir::Statement>),
}

impl State {
    fn start(label: operand::Label) -> Self {
        State::Block(label, Vec::new())
    }

    fn push(&mut self, statement: lir::Statement) {
        match self {
            State::Unreachable => (),
            State::Block(_, statements) => statements.push(statement),
        }
    }

    fn replace(&mut self, state: State) -> Option<(operand::Label, Vec<lir::Statement>)> {
        match mem::replace(self, state) {
            State::Unreachable => None,
            State::Block(label, statements) => Some((label, statements)),
        }
    }
}

fn construct_function(function: &lir::Function) -> Control {
    let mut graph = DiGraphMap::new();
    let mut blocks = BTreeMap::new();

    let start = operand::Label::fresh("start");

    let mut block = State::Block(start, Vec::new());

    for statement in &function.statements {
        match statement {
            lir::Statement::Jump(target) => {
                block.push(lir::Statement::Jump(*target));

                if let Some((label, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(label, *target, ());
                    blocks.insert(label, statements);
                }
            }
            lir::Statement::CJump(expression, r#true, r#false) => {
                block.push(lir::Statement::CJump(expression.clone(), *r#true, *r#false));

                if let Some((label, statements)) = block.replace(State::Unreachable) {
                    graph.add_edge(label, *r#true, ());
                    graph.add_edge(label, *r#false, ());
                    blocks.insert(label, statements);
                }
            }
            lir::Statement::Label(next) => {
                if let Some((previous, mut statements)) = block.replace(State::start(*next)) {
                    statements.push(lir::Statement::Jump(*next));
                    graph.add_edge(previous, *next, ());
                    blocks.insert(previous, statements);
                }
            }
            lir::Statement::Return(returns) => {
                block.push(lir::Statement::Return(returns.clone()));

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
