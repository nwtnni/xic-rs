use std::fmt;

use crate::cfg::Cfg;
use crate::cfg::Function;
use crate::data::ir;

impl<T> fmt::Display for ir::Unit<Cfg<T>>
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

impl<T> fmt::Display for Cfg<T>
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
