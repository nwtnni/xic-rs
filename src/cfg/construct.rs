use std::collections::BTreeMap;
use std::mem;

use petgraph::graphmap::DiGraphMap;

use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::cfg::Function;
use crate::cfg::Terminator;
use crate::data::asm;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Label;

pub fn construct_lir(unit: &lir::Unit<lir::Label>) -> ir::Unit<Cfg<lir::Function<lir::Label>>> {
    unit.map(|function| Walker::new().walk(function))
}

pub fn construct_assembly<T: Clone>(unit: &asm::Unit<T>) -> ir::Unit<Cfg<asm::Function<T>>> {
    unit.map(|function| Walker::new().walk(function))
}

struct Walker<T: Function> {
    enter: Label,
    exit: Label,
    graph: DiGraphMap<Label, Edge>,
    blocks: BTreeMap<Label, Vec<T::Statement>>,
    block: Block<T::Statement>,
}

impl<T: Function> Walker<T> {
    fn new() -> Self {
        let enter = Label::fresh("enter");
        let exit = Label::fresh("exit");

        let mut blocks = BTreeMap::new();
        blocks.insert(exit, Vec::new());

        Walker {
            enter,
            exit,
            graph: DiGraphMap::new(),
            blocks,
            block: Block::Reachable(enter, Vec::new()),
        }
    }

    fn walk(mut self, function: &T) -> Cfg<T> {
        let statements = function.statements();

        for (index, statement) in statements.iter().enumerate() {
            self.block.push(statement.clone());

            let terminator = match T::to_terminator(statement) {
                Some(terminator) => terminator,
                None => continue,
            };

            match terminator {
                Terminator::Label(label) => {
                    self.block.pop();
                    self.block.push(T::jump(label));
                    self.pop_unconditional(Block::reachable(label), label);
                }
                Terminator::Jump(label) => {
                    self.pop_unconditional(Block::Unreachable, label);
                }
                Terminator::CJump {
                    r#true,
                    r#false: Some(r#false),
                } => {
                    self.pop_conditional(Block::Unreachable, r#true, r#false);
                }
                Terminator::CJump {
                    r#true,
                    r#false: None,
                } => match statements.get(index + 1).map(T::to_terminator) {
                    None => self.pop_conditional(Block::Unreachable, r#true, self.exit),
                    Some(Some(Terminator::Label(label))) => {
                        self.pop_conditional(Block::Unreachable, r#true, label);
                    }
                    Some(_) => {
                        let fallthrough = Label::fresh("fallthrough");
                        self.pop_conditional(Block::reachable(fallthrough), r#true, fallthrough);
                    }
                },
                Terminator::Return => {
                    // IR return takes arguments, but assembly return does not. In order to
                    // simplify assembly tiling, we put this jump here and omit the actual `ret`
                    // instruction when tiling, so we can have a single `ret` in the epilogue:
                    //
                    // ```text
                    // (LABEL foo)
                    // (RETURN (CONST 1))
                    // (LABEL bar)
                    // (JUMP exit)
                    // (RETURN (CONST 2))
                    // (JUMP exit)
                    // (LABEL exit)
                    // ```
                    //
                    // ```
                    // foo:
                    //   mov rax, 1
                    //   jmp exit
                    // bar:
                    //   mov rax, 2
                    //   jmp exit
                    // exit:
                    //   ret
                    // ```
                    self.block.push(T::jump(self.exit));
                    self.pop_unconditional(Block::Unreachable, self.exit);
                }
            }
        }

        self.block.push(T::jump(self.exit));
        self.pop_unconditional(Block::Unreachable, self.exit);

        Cfg {
            name: function.name(),
            metadata: function.metadata(),
            enter: self.enter,
            exit: self.exit,
            graph: self.graph,
            blocks: self.blocks,
        }
    }

    fn pop_conditional(&mut self, replacement: Block<T::Statement>, r#true: Label, r#false: Label) {
        if let Some((label, statements)) = self.block.swap(replacement) {
            self.graph.add_edge(label, r#true, Edge::Conditional(true));
            self.graph
                .add_edge(label, r#false, Edge::Conditional(false));
            self.blocks.insert(label, statements);
        }
    }

    fn pop_unconditional(&mut self, replacement: Block<T::Statement>, target: Label) {
        if let Some((label, statements)) = self.block.swap(replacement) {
            self.graph.add_edge(label, target, Edge::Unconditional);
            self.blocks.insert(label, statements);
        }
    }
}

enum Block<T> {
    Unreachable,
    Reachable(Label, Vec<T>),
}

impl<T> Block<T> {
    fn reachable(label: Label) -> Self {
        Block::Reachable(label, Vec::new())
    }

    fn push(&mut self, statement: T) {
        match self {
            Block::Unreachable => (),
            Block::Reachable(_, statements) => statements.push(statement),
        }
    }

    fn pop(&mut self) {
        match self {
            Block::Unreachable => (),
            Block::Reachable(_, statements) => {
                statements.pop();
            }
        }
    }

    fn swap(&mut self, replacement: Block<T>) -> Option<(Label, Vec<T>)> {
        match mem::replace(self, replacement) {
            Block::Unreachable => None,
            Block::Reachable(label, statements) => Some((label, statements)),
        }
    }
}
