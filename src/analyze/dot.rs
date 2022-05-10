use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;

use crate::analyze::Analysis;
use crate::analyze::Solution;
use crate::cfg::Cfg;
use crate::cfg::Dot;
use crate::cfg::Function;

pub fn display<'cfg, A, T>(solution: &'cfg Solution<A, T>, cfg: &'cfg Cfg<T>) -> Dot<'cfg, T>
where
    A: Analysis<T> + 'cfg,
    A::Data: Display,
    T: Function + 'cfg,
    T::Statement: fmt::Display,
{
    Dot::new(cfg, move |label, statements| {
        let mut output = solution.inputs[label].clone();
        let mut outputs = vec![(&output as &dyn Display).to_string()];

        if A::BACKWARD {
            for (index, statement) in statements.iter().enumerate().rev() {
                solution
                    .analysis
                    .transfer_with_metadata(label, index, statement, &mut output);
                outputs.push((&output as &dyn Display).to_string());
            }
            outputs.reverse();
        } else {
            for (index, statement) in statements.iter().enumerate() {
                solution
                    .analysis
                    .transfer_with_metadata(label, index, statement, &mut output);
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

impl<K: fmt::Display, V: fmt::Display> Display for BTreeMap<K, V> {
    fn format(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{{")?;

        let mut iter = self.iter();

        if let Some((key, value)) = iter.next() {
            write!(fmt, "{}: {}", key, value)?;
        }

        for (key, value) in iter {
            write!(fmt, ", {}: {}", key, value)?;
        }

        write!(fmt, "}}")
    }
}

impl fmt::Display for &dyn Display {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.format(fmt)
    }
}
