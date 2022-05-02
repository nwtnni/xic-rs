use std::fmt;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::Solution;
use crate::cfg::Cfg;
use crate::cfg::Function;
use crate::data::ir;

pub fn display<'cfg, A, T>(cfg: &'cfg Cfg<T>) -> impl fmt::Display + 'cfg
where
    A: Analysis<T> + 'cfg,
    A::Data: fmt::Display,
    T: Function + 'cfg,
    T::Statement: fmt::Display,
{
    let (analysis, solution) = analyze::<A, T>(cfg);
    Dot {
        analysis,
        cfg,
        solution,
    }
}

struct Dot<'cfg, A: Analysis<T>, T: Function> {
    analysis: A,
    cfg: &'cfg Cfg<T>,
    solution: Solution<A, T>,
}

impl<'cfg, A, T> fmt::Display for ir::Unit<Dot<'cfg, A, T>>
where
    A: Analysis<T>,
    A::Data: fmt::Display,
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

impl<'cfg, A, T> fmt::Display for Dot<'cfg, A, T>
where
    A: Analysis<T>,
    A::Data: fmt::Display,
    T: Function,
    T::Statement: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "  subgraph cluster_{} {{", self.cfg.name())?;
        writeln!(fmt, "    label=\"{}\"", self.cfg.name())?;

        for (label, statements) in self.cfg.blocks() {
            write!(fmt, "    \"{0}\" [label=\"\\\n{0}:\\l", label)?;

            let mut output = self.solution.inputs[label].clone();

            write!(
                fmt,
                "\\\n    {}\\l",
                output.to_string().replace('\n', "\\l\\\n    ")
            )?;

            for statement in statements {
                write!(
                    fmt,
                    "\\\n    {};\\l",
                    statement.to_string().replace('\n', "\\l\\\n    ")
                )?;

                self.analysis.transfer(statement, &mut output);

                write!(
                    fmt,
                    "\\\n    {}\\l",
                    output.to_string().replace('\n', "\\l\\\n    ")
                )?;
            }

            writeln!(fmt, "  \"];")?;
        }

        let mut edges = self.cfg.edges().collect::<Vec<_>>();

        edges.sort();

        for (from, to, _) in edges {
            writeln!(fmt, r#"    "{}" -> "{}";"#, from, to)?;
        }

        writeln!(fmt, "  }}")
    }
}
