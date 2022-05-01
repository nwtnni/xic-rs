use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::mem;

use petgraph::graphmap::DiGraphMap;

use crate::data::asm;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Label;
use crate::data::symbol::Symbol;

pub struct Control<T: Function> {
    name: Symbol,
    metadata: T::Metadata,
    enter: Label,
    #[allow(dead_code)]
    exit: Label,
    graph: DiGraphMap<Label, Edge>,
    blocks: BTreeMap<Label, Vec<T::Statement>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Edge {
    Unconditional,
    Conditional(bool),
}

pub fn construct_lir(unit: &lir::Unit<lir::Label>) -> ir::Unit<Control<lir::Function<lir::Label>>> {
    unit.map(|function| Walker::new().walk(function))
}

pub fn construct_assembly<T: Clone>(unit: &asm::Unit<T>) -> ir::Unit<Control<asm::Function<T>>> {
    unit.map(|function| Walker::new().walk(function))
}

pub fn destruct_lir(
    unit: &ir::Unit<Control<lir::Function<lir::Label>>>,
) -> lir::Unit<lir::Fallthrough> {
    unit.map(|function| {
        let statements = destruct_function(function)
            .into_iter()
            .map(fallthrough)
            .collect();
        let (arguments, returns) = function.metadata;
        lir::Function {
            name: function.name,
            statements,
            arguments,
            returns,
        }
    })
}

pub fn destruct_assembly<T: Clone>(unit: &ir::Unit<Control<asm::Function<T>>>) -> asm::Unit<T> {
    unit.map(|function| {
        let instructions = destruct_function(function);
        let (arguments, returns, callee_arguments, callee_returns) = function.metadata;
        asm::Function {
            name: function.name,
            instructions,
            arguments,
            returns,
            callee_arguments,
            callee_returns,
        }
    })
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

    fn walk(mut self, function: &T) -> Control<T> {
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

        Control {
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

/// After linearization, guarantees that `function.enter` is at the beginning of the
/// list, and that `function.exit` is at the end. This property is useful for tiling
/// assembly, so we can place the function prologue and epilogue accurately.
///
/// Also guarantees that conditional jumps are immediately followed by their false branch.
fn destruct_function<T: Function>(function: &Control<T>) -> Vec<T::Statement> {
    let mut dfs = vec![function.enter];
    let mut statements = Vec::new();
    let mut visited = BTreeSet::new();

    while let Some(label) = dfs.pop() {
        if !visited.insert(label) {
            continue;
        }

        if label == function.exit {
            continue;
        }

        statements.push(T::label(label));
        statements.extend_from_slice(&function.blocks[&label]);

        let mut conditional = [None; 2];

        for (_, next, edge) in function.graph.edges(label) {
            match edge {
                Edge::Unconditional => dfs.push(next),
                Edge::Conditional(true) => conditional[0] = Some(next),
                Edge::Conditional(false) if !visited.contains(&next) => conditional[1] = Some(next),
                Edge::Conditional(false) => statements.push(T::jump(next)),
            }
        }

        dfs.extend(IntoIterator::into_iter(conditional).flatten());
    }

    statements.push(T::label(function.exit));
    statements.extend_from_slice(&*function.blocks[&function.exit]);
    statements
}

impl<T> fmt::Display for ir::Unit<Control<T>>
where
    T: Function,
    T::Statement: fmt::Display,
{
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

impl<T> fmt::Display for Control<T>
where
    T: Function,
    T::Statement: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "  subgraph cluster_{} {{", self.name)?;
        writeln!(fmt, "    label=\"{}\"", self.name)?;

        for (label, statements) in &self.blocks {
            write!(fmt, "    \"{0}\" [label=\"\\\n{0}:\\l", label)?;

            for statement in statements {
                write!(
                    fmt,
                    "\\\n    {};\\l",
                    statement.to_string().replace('\n', "\\l\\\n    ")
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

fn fallthrough(statement: lir::Statement<lir::Label>) -> lir::Statement<lir::Fallthrough> {
    use lir::Statement::*;
    match statement {
        Jump(label) => Jump(label),
        CJump {
            condition,
            left,
            right,
            r#true,
            r#false: _,
        } => CJump {
            condition,
            left,
            right,
            r#true,
            r#false: lir::Fallthrough,
        },
        Call(function, arguments, returns) => Call(function, arguments, returns),
        Label(label) => Label(label),
        Move {
            destination,
            source,
        } => Move {
            destination,
            source,
        },
        Return(returns) => Return(returns),
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
        asm::Assembly::Unary(
            asm::Unary::Jmp,
            operand::Unary::I(operand::Immediate::Label(label)),
        )
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
            asm::Assembly::Label(label) => Some(Terminator::Label(*label)),
            asm::Assembly::Unary(
                asm::Unary::Jmp,
                operand::Unary::I(operand::Immediate::Label(label)),
            ) => Some(Terminator::Jump(*label)),
            asm::Assembly::Unary(asm::Unary::Jmp, _) => unreachable!(),
            asm::Assembly::Unary(
                asm::Unary::Jcc(_),
                operand::Unary::I(operand::Immediate::Label(label)),
            ) => Some(Terminator::CJump {
                r#true: *label,
                r#false: None,
            }),
            asm::Assembly::Unary(asm::Unary::Jcc(_), _) => unreachable!(),
            asm::Assembly::Unary(_, _) => None,
        }
    }
}
