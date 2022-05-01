use std::collections::BTreeSet;

use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::cfg::Function;
use crate::data::asm;
use crate::data::ir;
use crate::data::lir;

pub fn destruct_lir(
    unit: &ir::Unit<Cfg<lir::Function<lir::Label>>>,
) -> lir::Unit<lir::Fallthrough> {
    unit.map(|function| {
        let statements = destruct_function(function)
            .into_iter()
            .map(lir::Statement::lower)
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

pub fn destruct_assembly<T: Clone>(unit: &ir::Unit<Cfg<asm::Function<T>>>) -> asm::Unit<T> {
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

/// After linearization, guarantees that `function.enter` is at the beginning of the
/// list, and that `function.exit` is at the end. This property is useful for tiling
/// assembly, so we can place the function prologue and epilogue accurately.
///
/// Also guarantees that conditional jumps are immediately followed by their false branch.
fn destruct_function<T: Function>(function: &Cfg<T>) -> Vec<T::Statement> {
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
