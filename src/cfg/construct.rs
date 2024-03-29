use std::mem;

use petgraph::graphmap::DiGraphMap;

use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::cfg::Function;
use crate::cfg::Terminator;
use crate::data::operand::Label;
use crate::util;
use crate::Map;

pub fn construct_cfg<T: Function>(function: T) -> Cfg<T> {
    log::info!(
        "[{}] Constructing CFG for {}...",
        std::any::type_name::<T>(),
        function.name(),
    );
    util::time!(
        "[{}] Done constructing CFG for {}",
        std::any::type_name::<T>(),
        function.name(),
    );
    Walker::new(&function).walk(function)
}

struct Walker<T: Function> {
    enter: Label,
    exit: Label,
    graph: DiGraphMap<Label, Edge>,
    blocks: Map<Label, Vec<T::Statement>>,
    block: Block<T::Statement>,
}

impl<T: Function> Walker<T> {
    fn new(function: &T) -> Self {
        let (enter, block) = match function.enter() {
            Some(enter) => (*enter, Block::Unreachable),
            None => {
                let enter = Label::fresh("enter");
                (enter, Block::reachable(enter))
            }
        };

        let (exit, blocks) = match function.exit() {
            Some(exit) => (*exit, Map::default()),
            None => {
                let exit = Label::fresh("exit");
                let mut blocks = Map::default();
                blocks.insert(exit, Vec::new());
                (exit, blocks)
            }
        };

        Walker {
            enter,
            exit,
            graph: DiGraphMap::default(),
            blocks,
            block,
        }
    }

    fn walk(mut self, mut function: T) -> Cfg<T> {
        let mut statements = function.statements().into_iter().peekable();

        while let Some(statement) = statements.next() {
            let terminator = T::to_terminator(&statement);

            self.block.push(statement);

            let terminator = match terminator {
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
                } => match statements.peek().map(T::to_terminator) {
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
                    // simplify assembly tiling, we (1) put this jump here and (2) omit the
                    // actual `ret` statement when tiling, so we can have a single `ret`
                    // in the epilogue:
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

        // Handle exit block if it exists, or jump to the generated one.
        if let Some((label, mut statements)) = self.block.swap(Block::Unreachable) {
            if label != self.exit {
                statements.push(T::jump(self.exit));
                self.graph.add_edge(label, self.exit, Edge::Unconditional);
            }

            self.blocks.insert(label, statements);
        }

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
