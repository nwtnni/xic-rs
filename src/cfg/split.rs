use petgraph::Direction;

use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::cfg::Function;
use crate::cfg::TerminatorMut;
use crate::data::operand::Label;
use crate::util;

/// Split critical edges, which are edges from a block with multiple successors to a block with
/// multiple predecessors.
pub fn split_cfg<T: Function>(cfg: &mut Cfg<T>) {
    log::info!(
        "[{}] Splitting critical edges in {}...",
        std::any::type_name::<Cfg<T>>(),
        cfg.name(),
    );
    util::time!(
        "[{}] Done splitting critical edges in {}",
        std::any::type_name::<Cfg<T>>(),
        cfg.name(),
    );

    let split = cfg
        .graph
        .all_edges()
        .filter(|(predecessor, successor, _)| {
            cfg.graph
                .neighbors_directed(*predecessor, Direction::Outgoing)
                .count()
                > 1
                && cfg
                    .graph
                    .neighbors_directed(*successor, Direction::Incoming)
                    .count()
                    > 1
        })
        .map(|(predecessor, successor, edge)| (predecessor, successor, *edge))
        .collect::<Vec<_>>();

    for (predecessor, successor, edge) in split {
        let split = Label::fresh("split");

        cfg.blocks.insert(split, vec![T::jump(successor)]);
        cfg.graph.add_edge(predecessor, split, edge);
        cfg.graph.add_edge(split, successor, Edge::Unconditional);
        cfg.graph.remove_edge(predecessor, successor);

        let terminator = cfg
            .blocks
            .get_mut(&predecessor)
            .and_then(|statements| statements.last_mut())
            .and_then(T::to_terminator_mut)
            .unwrap();

        match (edge, terminator) {
            (Edge::Unconditional, TerminatorMut::Jump(target)) => {
                *target = split;
            }
            (Edge::Conditional(true), TerminatorMut::CJump { r#true, r#false: _ }) => {
                *r#true = split;
            }
            (
                Edge::Conditional(false),
                TerminatorMut::CJump {
                    r#true: _,
                    r#false: Some(r#false),
                },
            ) => {
                *r#false = split;
            }
            (
                Edge::Conditional(false),
                TerminatorMut::CJump {
                    r#true: _,
                    r#false: None,
                },
            ) => (),
            (Edge::Unconditional, TerminatorMut::CJump { .. })
            | (Edge::Conditional(_), TerminatorMut::Jump(_)) => unreachable!(),
        }
    }
}
