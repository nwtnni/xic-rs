use std::collections::BTreeSet;
use std::mem;

use petgraph::Direction;

use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::cfg::Function;
use crate::cfg::Terminator;
use crate::cfg::TerminatorMut;
use crate::data::operand::Label;

pub fn clean_cfg<T: Function>(cfg: &mut Cfg<T>) {
    let mut stack = Vec::new();
    let mut order = Vec::new();
    let mut visited = BTreeSet::new();
    let mut dirty = true;

    let mut buffer = Vec::new();

    while mem::take(&mut dirty) {
        postorder(cfg, &mut visited, &mut order, &mut stack);

        buffer.extend(cfg.graph.nodes().filter(|label| !visited.contains(label)));

        for unreachable in buffer.drain(..) {
            cfg.graph.remove_node(unreachable);
            cfg.blocks.remove(&unreachable);
        }

        for label in order.drain(..) {
            let terminator = match cfg.get_terminator(&label) {
                Some(terminator) => terminator,
                None => continue,
            };

            let target = match terminator {
                Terminator::Jump(target) => target,
                _ => continue,
            };

            // Block is empty: rewrite predecessors to jump directly to `target`
            if cfg.blocks[&label].len() == 1 {
                buffer.extend(cfg.incoming(&label));

                for predecessor in buffer.drain(..) {
                    match cfg.graph.remove_edge(predecessor, label).unwrap() {
                        Edge::Unconditional => {
                            cfg.graph.add_edge(predecessor, target, Edge::Unconditional);

                            match cfg.get_terminator_mut(&predecessor).unwrap() {
                                TerminatorMut::Jump(label) => *label = target,
                                TerminatorMut::CJump { .. } => unreachable!(),
                            }
                        }
                        Edge::Conditional(branch) => {
                            cfg.graph
                                .add_edge(predecessor, target, Edge::Conditional(branch));

                            match cfg.get_terminator_mut(&predecessor).unwrap() {
                                TerminatorMut::Jump(_) => unreachable!(),
                                TerminatorMut::CJump { r#true, r#false } => {
                                    if branch {
                                        *r#true = target;
                                    } else if let Some(r#false) = r#false {
                                        *r#false = target;
                                    }
                                }
                            };
                        }
                    }
                }

                dirty = true;
            }

            // Combine block with successor
            if cfg.incoming(&target).count() == 1 && target != cfg.exit {
                cfg.get_mut(&label).and_then(|block| block.pop());

                let mut block = cfg.blocks.remove(&target).unwrap();
                cfg.get_mut(&label).unwrap().append(&mut block);

                buffer.extend(cfg.outgoing(&target));

                for successor in buffer.drain(..) {
                    let edge = cfg.graph.remove_edge(target, successor).unwrap();
                    cfg.graph.add_edge(label, successor, edge);
                }

                assert!(cfg.graph.remove_node(target));
                dirty = true;
            }
        }
    }
}

enum Event {
    Push(Label),
    Pop(Label),
}

fn postorder<T: Function>(
    cfg: &Cfg<T>,
    visited: &mut BTreeSet<Label>,
    order: &mut Vec<Label>,
    stack: &mut Vec<Event>,
) {
    visited.clear();

    stack.push(Event::Push(cfg.enter));
    visited.insert(cfg.enter);

    while let Some(event) = stack.pop() {
        match event {
            Event::Push(label) => {
                stack.push(Event::Pop(label));
                cfg.graph
                    .neighbors_directed(label, Direction::Outgoing)
                    .filter(|successor| visited.insert(*successor))
                    .for_each(|successor| stack.push(Event::Push(successor)));
            }
            Event::Pop(label) => order.push(label),
        }
    }
}
