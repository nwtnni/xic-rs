use std::fmt;

use crate::cfg::Cfg;
use crate::cfg::Function;
use crate::data::ir;
use crate::data::operand::Label;

pub struct Dot<'cfg, T: Function> {
    cfg: &'cfg Cfg<T>,
    #[allow(clippy::type_complexity)]
    format: Box<dyn Fn(&'cfg Label, &'cfg [T::Statement]) -> Result<String, fmt::Error> + 'cfg>,
}

impl<'cfg, T: Function> Dot<'cfg, T> {
    pub fn new<F>(cfg: &'cfg Cfg<T>, format: F) -> Self
    where
        F: Fn(&'cfg Label, &'cfg [T::Statement]) -> Result<String, fmt::Error> + 'cfg,
    {
        Dot {
            cfg,
            format: Box::new(format),
        }
    }
}

impl<'cfg, T> fmt::Display for ir::Unit<Dot<'cfg, T>>
where
    T: Function,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "digraph {{")?;
        writeln!(fmt, "    label=\"{}\"", self.name)?;
        writeln!(fmt, "    node [shape=box nojustify=true]")?;

        for function in self.functions.values() {
            writeln!(fmt, "{:#}", function)?;
        }

        writeln!(fmt, "}}")
    }
}

impl<'cfg, T> fmt::Display for Dot<'cfg, T>
where
    T: Function,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut indent = String::from("");
        let mut escape = String::from("\\l\\\n");

        if fmt.alternate() {
            indent.push_str("    ");
            escape.push_str("    ");
            writeln!(fmt, "    subgraph cluster_{} {{", self.cfg.name)?;
        } else {
            writeln!(fmt, "digraph {} {{", self.cfg.name)?;
        };

        writeln!(fmt, "{}    label=\"{}\"", indent, self.cfg.name)?;

        for (label, statements) in &self.cfg.blocks {
            write!(fmt, "{0}    \"{1}\" [label=\"", indent, label)?;

            let statements = (self.format)(label, statements)?
                .trim()
                .replace('\n', &escape)
                .replace('"', "\\\"");

            if statements.is_empty() {
                writeln!(fmt, "{}:\"];", label)?;
            } else {
                writeln!(fmt, "\\\n{}:{}{}\"];", label, escape, statements)?;
            }
        }

        let mut edges = self.cfg.graph.all_edges().collect::<Vec<_>>();

        edges.sort();

        for (from, to, _) in edges {
            writeln!(fmt, r#"{}    "{}" -> "{}";"#, indent, from, to)?;
        }

        writeln!(fmt, "{}}}", indent)
    }
}
