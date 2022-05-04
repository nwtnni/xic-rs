use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::Direction as _;
use crate::cfg::Cfg;
use crate::cfg::Dot;
use crate::cfg::Function;

pub fn display<'cfg, A, T>(cfg: &'cfg Cfg<T>) -> Dot<'cfg, T>
where
    A: Analysis<T> + 'cfg,
    A::Data: Display,
    T: Function + 'cfg,
    T::Statement: fmt::Display,
{
    let (analysis, solution) = analyze::<A, T>(cfg);

    Dot::new(cfg, move |label, statements| {
        let mut output = solution.inputs[label].clone();
        let mut outputs = vec![(&output as &dyn Display).to_string()];

        if A::Direction::REVERSE {
            for statement in statements.iter().rev() {
                analysis.transfer(statement, &mut output);
                outputs.push((&output as &dyn Display).to_string());
            }
            outputs.reverse();
        } else {
            for statement in statements {
                analysis.transfer(statement, &mut output);
                outputs.push((&output as &dyn Display).to_string());
            }
        }

        let mut outputs = outputs.into_iter();
        let mut string = outputs.next().unwrap();

        for statement in statements {
            write!(&mut string, "\n{}\n{}", statement, outputs.next().unwrap())?;
        }

        Ok(string)
    })
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
