mod construct;
mod destruct;
mod dot;

pub use construct::unit as construct_cfg;
pub use destruct::unit as destruct_cfg;
pub(crate) use dot::Dot;

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;
use std::ops;

use petgraph::algo;
use petgraph::graphmap::DiGraphMap;
use petgraph::graphmap::NeighborsDirected;
use petgraph::visit;

use crate::data::asm;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
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
    pub fn name(&self) -> &Symbol {
        &self.name
    }

    pub fn enter(&self) -> &Label {
        &self.enter
    }

    pub fn exit(&self) -> &Label {
        &self.exit
    }

    /// Computes the strongly connected components (SCCs) of this (`reverse`d) control flow graph.
    ///
    /// The list of SCCs is in reverse topological order. Moreover, the list of labels in each
    /// SCC is in postorder, with the component root last.
    pub fn strongly_connected_components(&self, reverse: bool) -> Vec<Vec<Label>> {
        // Wikipedia says:
        //
        // > ... there is nothing special about the order of the nodes within
        // > each strongly connected component...
        //
        // and the `petgraph::algo::tarjan_scc` documentation says:
        //
        // > The order of node ids within each scc is arbitrary...
        //
        // But as far as I can tell from reading the algorithm and source code,
        // the SCCs are generated by postorder traversal?
        if reverse {
            algo::tarjan_scc(visit::Reversed(&self.graph))
        } else {
            algo::tarjan_scc(&self.graph)
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item = (&Label, &[T::Statement])> {
        self.blocks
            .iter()
            .map(|(label, statement)| (label, statement.as_slice()))
    }

    pub fn edges(&self) -> impl Iterator<Item = (Label, Label, &Edge)> {
        self.graph.all_edges()
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

pub fn display_cfg<T>(unit: &ir::Unit<Cfg<T>>) -> impl fmt::Display + '_
where
    T: Function,
    T::Statement: fmt::Display,
{
    unit.map(|cfg| {
        dot::Dot::new(cfg, |_, statements| {
            let mut string = String::new();
            for statement in statements {
                writeln!(&mut string, "{}", statement)?;
            }
            Ok(string)
        })
    })
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
    fn exit(&self) -> Vec<Self::Statement>;

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

    fn exit(&self) -> Vec<Self::Statement> {
        Vec::new()
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

impl Function for asm::Function<Temporary> {
    type Statement = asm::Assembly<Temporary>;
    type Metadata = (usize, usize, usize, usize, Option<Temporary>);
    type Fallthrough = asm::Function<Temporary>;

    fn new(
        name: Symbol,
        mut instructions: Vec<Self::Statement>,
        (arguments, returns, callee_arguments, callee_returns, caller_returns): Self::Metadata,
    ) -> Self {
        // Note: we want to maintain some invariants:
        //
        // (1) `asm::Function<Temporary>` never contains a `ret` instruction
        // (2) `cfg::Cfg<asm::Function<Temporary>>` contains a single `ret` instruction in its exit block
        //
        // Because `destruct_cfg` guarantees that the exit block will be at the end,
        // we can preserve these two invariants across CFG round-trips by popping here.
        assert_eq!(
            instructions.pop(),
            Some(asm::Assembly::Nullary(asm::Nullary::Ret(
                returns,
                caller_returns
            )))
        );

        asm::Function {
            name,
            instructions,
            arguments,
            returns,
            callee_arguments,
            callee_returns,
            caller_returns,
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
            self.caller_returns,
        )
    }

    fn statements(&self) -> &[Self::Statement] {
        &self.instructions
    }

    fn exit(&self) -> Vec<Self::Statement> {
        vec![asm::Assembly::Nullary(asm::Nullary::Ret(
            self.returns,
            self.caller_returns,
        ))]
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
            asm::Assembly::Nullary(asm::Nullary::Ret(_, _)) => {
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
