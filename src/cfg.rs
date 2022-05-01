mod construct;
mod destruct;
mod dot;

pub use construct::unit as construct_cfg;
pub use destruct::unit as destruct_cfg;

use std::collections::BTreeMap;
use std::ops;

use petgraph::graphmap::DiGraphMap;
use petgraph::graphmap::NeighborsDirected;

use crate::data::asm;
use crate::data::lir;
use crate::data::operand::Label;
use crate::data::symbol::Symbol;

pub struct Cfg<T: Function> {
    name: Symbol,
    metadata: T::Metadata,
    enter: Label,
    exit: Label,
    graph: DiGraphMap<Label, Edge>,
    blocks: BTreeMap<Label, Vec<T::Statement>>,
}

impl<T: Function> Cfg<T> {
    pub fn enter(&self) -> &Label {
        &self.enter
    }

    pub fn exit(&self) -> &Label {
        &self.exit
    }

    pub fn incoming(&self, label: &Label) -> NeighborsDirected<Label, petgraph::Directed> {
        self.graph
            .neighbors_directed(*label, petgraph::Direction::Incoming)
    }

    pub fn outgoing(&self, label: &Label) -> NeighborsDirected<Label, petgraph::Directed> {
        self.graph
            .neighbors_directed(*label, petgraph::Direction::Outgoing)
    }

    pub fn get(&self, label: &Label) -> Option<&[T::Statement]> {
        self.blocks.get(label).map(|block| block.as_slice())
    }
}

impl<T: Function> ops::Index<&Label> for Cfg<T> {
    type Output = [T::Statement];
    fn index(&self, index: &Label) -> &Self::Output {
        &self.blocks[index]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Edge {
    Unconditional,
    Conditional(bool),
}

/// Represents a type that can be converted to and from a control flow graph.
pub trait Function {
    type Statement: Clone;
    type Metadata: Clone;
    type Fallthrough;

    fn new(
        name: Symbol,
        statements: Vec<Self::Statement>,
        metadata: Self::Metadata,
    ) -> Self::Fallthrough;
    fn name(&self) -> Symbol;
    fn metadata(&self) -> Self::Metadata;
    fn statements(&self) -> &[Self::Statement];
    fn jump(label: Label) -> Self::Statement;
    fn label(label: Label) -> Self::Statement;
    fn to_terminator(instruction: &Self::Statement) -> Option<Terminator>;
}

pub enum Terminator {
    Label(Label),
    Jump(Label),
    CJump {
        r#true: Label,
        r#false: Option<Label>,
    },
    Return,
}

impl<T: lir::Target + Clone> Function for lir::Function<T> {
    type Statement = lir::Statement<T>;
    type Metadata = (usize, usize);
    type Fallthrough = lir::Function<lir::Fallthrough>;

    fn new(
        name: Symbol,
        statements: Vec<Self::Statement>,
        (arguments, returns): Self::Metadata,
    ) -> Self::Fallthrough {
        lir::Function {
            name,
            statements: statements.into_iter().map(lir::Statement::lower).collect(),
            arguments,
            returns,
        }
    }

    fn name(&self) -> Symbol {
        self.name
    }

    fn metadata(&self) -> Self::Metadata {
        (self.arguments, self.returns)
    }

    fn statements(&self) -> &[Self::Statement] {
        &self.statements
    }

    fn jump(label: Label) -> Self::Statement {
        lir::Statement::Jump(label)
    }

    fn label(label: Label) -> Self::Statement {
        lir::Statement::Label(label)
    }

    fn to_terminator(instruction: &Self::Statement) -> Option<Terminator> {
        match instruction {
            lir::Statement::Jump(label) => Some(Terminator::Jump(*label)),
            lir::Statement::CJump {
                condition: _,
                left: _,
                right: _,
                r#true,
                r#false,
            } => Some(Terminator::CJump {
                r#true: *r#true,
                r#false: r#false.label().copied(),
            }),
            lir::Statement::Call(_, _, _) => None,
            lir::Statement::Label(label) => Some(Terminator::Label(*label)),
            lir::Statement::Move {
                destination: _,
                source: _,
            } => None,
            lir::Statement::Return(_) => Some(Terminator::Return),
        }
    }
}

impl<T: Clone> Function for asm::Function<T> {
    type Statement = asm::Assembly<T>;
    type Metadata = (usize, usize, usize, usize);
    type Fallthrough = asm::Function<T>;

    fn new(
        name: Symbol,
        instructions: Vec<Self::Statement>,
        (arguments, returns, callee_arguments, callee_returns): Self::Metadata,
    ) -> Self {
        asm::Function {
            name,
            instructions,
            arguments,
            returns,
            callee_arguments,
            callee_returns,
        }
    }

    fn name(&self) -> Symbol {
        self.name
    }

    fn metadata(&self) -> Self::Metadata {
        (
            self.arguments,
            self.returns,
            self.callee_arguments,
            self.callee_returns,
        )
    }

    fn statements(&self) -> &[Self::Statement] {
        &self.instructions
    }

    fn jump(label: Label) -> Self::Statement {
        asm::Assembly::Jmp(label)
    }

    fn label(label: Label) -> Self::Statement {
        asm::Assembly::Label(label)
    }

    fn to_terminator(instruction: &Self::Statement) -> Option<Terminator> {
        match instruction {
            asm::Assembly::Nullary(asm::Nullary::Cqo) => None,
            asm::Assembly::Nullary(asm::Nullary::Ret) => {
                unreachable!("no ret instruction until register allocation")
            }
            asm::Assembly::Binary(_, _) => None,
            asm::Assembly::Unary(_, _) => None,
            asm::Assembly::Label(label) => Some(Terminator::Label(*label)),
            asm::Assembly::Jmp(label) => Some(Terminator::Jump(*label)),
            asm::Assembly::Jcc(_, label) => Some(Terminator::CJump {
                r#true: *label,
                r#false: None,
            }),
        }
    }
}
