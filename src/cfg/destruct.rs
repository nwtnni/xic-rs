use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::cfg::Function;
use crate::cfg::Terminator;
use crate::util;

/// After linearization, guarantees that `function.enter` is at the beginning of the
/// list, and that `function.exit` is at the end. This property is useful for tiling
/// assembly, so we can place the function prologue and epilogue accurately.
///
/// Also guarantees that conditional jumps are immediately followed by their false branch.
pub fn destruct_cfg<T: Function>(mut function: Cfg<T>) -> T::Fallthrough {
    log::info!(
        "[{}] Destructing CFG for {}...",
        std::any::type_name::<Cfg<T>>(),
        function.name(),
    );
    util::time!(
        "[{}] Done destructing CFG for {}",
        std::any::type_name::<Cfg<T>>(),
        function.name(),
    );

    let mut dfs = vec![function.enter];
    let mut statements = Vec::new();

    while let Some(label) = dfs.pop() {
        if label == function.exit {
            continue;
        }

        let mut block = match function.blocks.remove(&label) {
            None => continue,
            Some(block) => block,
        };

        statements.push(T::label(label));
        statements.append(&mut block);

        let mut conditional = [None; 2];

        for (_, next, edge) in function.graph.edges(label) {
            // Special case: since we require that the exit block is the
            // last statement in the linearized function, we can't fall
            // through to it unless `label` is the penultimate block.
            match edge {
                Edge::Unconditional if !function.blocks.contains_key(&next) => (),
                Edge::Unconditional if next == function.exit && function.blocks.len() != 1 => {
                    dfs.push(next);
                }
                Edge::Unconditional => {
                    assert!(matches!(
                        statements
                            .pop()
                            .and_then(|statement| T::to_terminator(&statement)),
                        Some(Terminator::Jump(_))
                    ));
                    dfs.push(next);
                }

                Edge::Conditional(true) if !function.blocks.contains_key(&next) => (),
                Edge::Conditional(true) => conditional[0] = Some(next),

                Edge::Conditional(false) if !function.blocks.contains_key(&next) => {
                    statements.push(T::jump(next));
                }
                Edge::Conditional(false) if next != function.exit || function.blocks.len() == 1 => {
                    conditional[1] = Some(next);
                }
                Edge::Conditional(false) => {
                    statements.push(T::jump(next));
                    conditional[1] = Some(next);
                }
            }
        }

        dfs.extend(conditional.into_iter().flatten());
    }

    statements.push(T::label(function.exit));
    statements.append(
        &mut function
            .blocks
            .remove(&function.exit)
            .expect("Unreachable exit label: program does not terminate"),
    );

    for (label, _) in function.blocks() {
        log::trace!("Removed unreachable block {}", label);
    }

    log::debug!("Removed {} unreachable blocks", function.blocks.len());

    T::new(
        function.name,
        statements,
        function.metadata.clone(),
        function.enter,
        function.exit,
    )
}
