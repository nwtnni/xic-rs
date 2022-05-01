mod construct;
mod destruct;
mod dot;

pub use construct::construct_assembly as construct_control_flow_assembly;
pub use construct::construct_lir as construct_control_flow_lir;
pub use destruct::destruct_assembly as destruct_control_flow_assembly;
pub use destruct::destruct_lir as destruct_control_flow_lir;

use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Edge {
    Unconditional,
    Conditional(bool),
}

pub trait Function {
    type Statement: Clone;
    type Metadata;

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
