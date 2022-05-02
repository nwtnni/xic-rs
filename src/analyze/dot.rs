use std::collections::BTreeSet;
use std::fmt;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::Solution;
use crate::cfg::Cfg;
use crate::cfg::Function;
use crate::data::ir;

pub fn display<'cfg, A, T>(unit: &'cfg ir::Unit<Cfg<T>>) -> impl fmt::Display + 'cfg
where
    A: Analysis<T> + 'cfg,
    A::Data: Display,
    T: Function + 'cfg,
    T::Statement: fmt::Display,
{
    unit.map(|cfg| {
        let (analysis, solution) = analyze::<A, T>(cfg);
        Dot {
            analysis,
            cfg,
            solution,
        }
    })
}

struct Dot<'cfg, A: Analysis<T>, T: Function> {
    analysis: A,
    cfg: &'cfg Cfg<T>,
    solution: Solution<A, T>,
}

impl<'cfg, A, T> fmt::Display for ir::Unit<Dot<'cfg, A, T>>
where
    A: Analysis<T>,
    A::Data: Display,
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
    A::Data: Display,
    T: Function,
    T::Statement: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "  subgraph cluster_{} {{", self.cfg.name())?;
        writeln!(fmt, "    label=\"{}\"", self.cfg.name())?;

        for (label, statements) in self.cfg.blocks() {
            write!(fmt, "    \"{0}\" [label=\"\\\n{0}:\\l", label)?;

            let mut output = self
                .solution
                .inputs
                .get(label)
                .cloned()
                .unwrap_or_else(|| self.analysis.default(self.cfg, label))
                .clone();

            let mut data = if std::any::type_name::<A::Direction>().contains("Forward") {
                let mut data = vec![(&output as &dyn Display).to_string()];
                for statement in statements.iter().rev() {
                    self.analysis.transfer(statement, &mut output);
                    data.push((&output as &dyn Display).to_string());
                }
                data.into_iter()
            } else if std::any::type_name::<A::Direction>().contains("Backward") {
                let mut data = vec![(&output as &dyn Display).to_string()];
                for statement in statements.iter().rev() {
                    self.analysis.transfer(statement, &mut output);
                    data.push((&output as &dyn Display).to_string());
                }
                data.reverse();
                data.into_iter()
            } else {
                unreachable!()
            };

            write!(
                fmt,
                "\\\n        {}\\l",
                data.next().unwrap().replace('\n', "\\l\\\n    ")
            )?;

            for statement in statements {
                write!(
                    fmt,
                    "\\\n    {}\\l\\\n        {}\\l",
                    statement.to_string().replace('\n', "\\l\\\n    "),
                    data.next().unwrap().replace('\n', "\\l\\\n    ")
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

pub trait Display {
    fn format(&self, fmt: &mut fmt::Formatter) -> fmt::Result;
}

impl<T: fmt::Display> Display for BTreeSet<T> {
    fn format(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{{")?;

        let mut iter = self.iter();

        if let Some(next) = iter.next() {
            write!(fmt, "{}", next)?;
        }

        for next in iter {
            write!(fmt, ", {}", next)?;
        }

        write!(fmt, "}}")
    }
}

impl fmt::Display for &dyn Display {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.format(fmt)
    }
}
