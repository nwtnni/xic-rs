use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::cfg::Function;

/// After linearization, guarantees that `function.enter` is at the beginning of the
/// list, and that `function.exit` is at the end. This property is useful for tiling
/// assembly, so we can place the function prologue and epilogue accurately.
///
/// Also guarantees that conditional jumps are immediately followed by their false branch.
pub fn destruct_cfg<T: Function>(mut function: Cfg<T>) -> T::Fallthrough {
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
            match edge {
                Edge::Unconditional => dfs.push(next),
                Edge::Conditional(true) => conditional[0] = Some(next),
                Edge::Conditional(false) if function.blocks.contains_key(&next) => {
                    conditional[1] = Some(next)
                }
                Edge::Conditional(false) => statements.push(T::jump(next)),
            }
        }

        dfs.extend(conditional.into_iter().flatten());
    }

    statements.push(T::label(function.exit));
    statements.append(&mut function.blocks.remove(&function.exit).unwrap());

    T::new(
        function.name,
        statements,
        function.metadata.clone(),
        function.enter,
        function.exit,
    )
}
