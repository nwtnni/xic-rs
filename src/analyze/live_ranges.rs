use std::cmp;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;

use crate::abi;
use crate::analyze::Analysis as _;
use crate::analyze::LiveVariables;
use crate::analyze::Solution;
use crate::cfg;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::symbol;

pub struct LiveRanges {
    pub ranges: BTreeMap<Temporary, Range>,
    pub function: asm::Function<Temporary>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    pub start: usize,
    pub end: usize,
    pub clobbered: Clobbered,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Clobbered {
    Caller,
    RaxRdx,
    Rax,
    Rdx,
    None,
}

impl Clobbered {
    fn or(&self, other: &Self) -> Self {
        use Clobbered::*;
        match (self, other) {
            (None, None) => None,
            (None, Rax) | (Rax, None) => Rax,
            (None, Rdx) | (Rdx, None) => Rdx,
            (Rax, Rax) => Rax,
            (Rdx, Rdx) => Rdx,
            (Rax, Rdx) | (Rdx, Rax) => RaxRdx,
            (Caller, _) | (_, Caller) => Caller,
            (RaxRdx, _) | (_, RaxRdx) => RaxRdx,
        }
    }

    pub fn as_slice(&self) -> &[Register] {
        match self {
            Clobbered::Caller => abi::CALLER_SAVED,
            Clobbered::RaxRdx => &[Register::Rax, Register::Rdx],
            Clobbered::Rax => &[Register::Rax],
            Clobbered::Rdx => &[Register::Rdx],
            Clobbered::None => &[],
        }
    }
}

impl Range {
    fn new(index: usize) -> Self {
        Range {
            start: index,
            end: index,
            clobbered: Clobbered::None,
        }
    }
}

impl LiveRanges {
    pub fn new(
        live_variables: &Solution<
            LiveVariables<asm::Function<Temporary>>,
            asm::Function<Temporary>,
        >,
        cfg: Cfg<asm::Function<Temporary>>,
    ) -> Self {
        let function = cfg::destruct_cfg(cfg);

        // Walk backward through straight-line abstract assembly to find basic blocks
        let labels =
            function
                .statements
                .iter()
                .enumerate()
                .rev()
                .filter_map(|(index, statement)| match statement {
                    asm::Statement::Label(label) => Some((index, *label)),
                    _ => None,
                });

        let mut ranges = BTreeMap::<Temporary, Range>::new();

        // Index of the statement we're transfering over.
        //
        // Careful: this starts at +1 to capture the dataflow input at the end
        // of each block, before starting to transfer backwards for each statement.
        //
        // With this convention, a range's start is the statement *after* it is
        // first defined, and a range's end is the statement where it is first used.
        let mut index = function.statements.len();

        for (start, label) in labels {
            let mut output = live_variables.inputs[&label].clone();

            for temporary in &output {
                ranges
                    .entry(*temporary)
                    .and_modify(|range| range.start = index)
                    .or_insert_with(|| Range::new(index));
            }

            while index > start {
                index -= 1;

                let statement = &function.statements[index];

                live_variables.analysis.transfer(statement, &mut output);

                let clobbered = match statement {
                    // Allow caller-saved registers to be used across _xi_out_of_bounds
                    // calls, because it diverges anyway.
                    asm::Statement::Unary(
                        asm::Unary::Call { .. },
                        operand::Unary::I(Immediate::Label(Label::Fixed(label))),
                    ) if symbol::resolve(*label) == abi::XI_OUT_OF_BOUNDS => Clobbered::None,
                    asm::Statement::Unary(asm::Unary::Call { .. }, _) => Clobbered::Caller,
                    asm::Statement::Unary(asm::Unary::Mul | asm::Unary::Div, _) => Clobbered::Rdx,
                    asm::Statement::Unary(asm::Unary::Hul | asm::Unary::Mod, _) => Clobbered::Rax,
                    asm::Statement::Nullary(asm::Nullary::Cqo) => Clobbered::Rdx,
                    _ => Clobbered::None,
                };

                for temporary in &output {
                    ranges
                        .entry(*temporary)
                        .and_modify(|range| {
                            range.clobbered = range.clobbered.or(&clobbered);
                            range.start = index;
                        })
                        .or_insert_with(|| Range::new(index));
                }
            }
        }

        Self { ranges, function }
    }

    /// Maximum number of simultaneously live variables.
    pub fn width(&self) -> usize {
        let mut max_width = 0;
        let mut width = 0;
        for point in self.points().iter().rev() {
            width = if point.start { width + 1 } else { width - 1 };
            max_width = cmp::max(width, max_width);
        }
        assert_eq!(width, 0);
        max_width
    }

    fn points(&self) -> Vec<Point> {
        let mut points = self
            .ranges
            .iter()
            .flat_map(|(temporary, range)| {
                [
                    Point::start(*temporary, range.start),
                    Point::end(*temporary, range.end),
                ]
            })
            .collect::<Vec<_>>();

        points.sort();
        points
    }
}

// Used for computing width.
//
// Must be sorted first by (a) decreasing index, and then (b)
// by end points first. This is so we can (a) have fast in-order
// removal via `.pop()`, and (b) use an unsigned counter, since
// we will encounter starts before ends.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
struct Point {
    index: cmp::Reverse<usize>,
    start: bool,
    temporary: Temporary,
}

impl Point {
    fn start(temporary: Temporary, index: usize) -> Self {
        Point {
            index: cmp::Reverse(index),
            start: true,
            temporary,
        }
    }

    fn end(temporary: Temporary, index: usize) -> Self {
        Point {
            index: cmp::Reverse(index),
            start: false,
            temporary,
        }
    }
}

impl fmt::Display for LiveRanges {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // Maximum width of a temporary name, plus one space for padding
        let mut max_temporary_width = 0;
        let mut buffer = String::new();

        for temporary in self.ranges.keys() {
            buffer.clear();
            write!(&mut buffer, "{}", temporary).unwrap();
            max_temporary_width = cmp::max(max_temporary_width, buffer.len() + 1);
        }

        let mut points = self.points();
        let width = self.width();

        let mut used = vec![Option::<(Temporary, Range)>::None; width];
        let mut free = (0..width).rev().collect::<Vec<_>>();

        for (index, statement) in self.function.statements.iter().enumerate() {
            while let Some(point) = points.last() {
                match point.index.0.cmp(&index) {
                    cmp::Ordering::Less if point.start => unreachable!(),
                    // Remove expired end points
                    cmp::Ordering::Less => {
                        let index = used
                            .iter()
                            .position(|slot| matches!(slot, Some((temporary, _)) if *temporary == point.temporary))
                            .unwrap();
                        used[index].take();
                        free.push(index);
                        points.pop();
                    }
                    // Insert new start points
                    cmp::Ordering::Equal if point.start => {
                        let index = free.pop().unwrap();
                        let range = self.ranges[&point.temporary];
                        assert!(used[index].replace((point.temporary, range)).is_none());
                        points.pop();
                    }
                    cmp::Ordering::Equal | cmp::Ordering::Greater => break,
                }
            }

            for slot in &used {
                match slot {
                    None => write!(fmt, "{:1$}", "", max_temporary_width)?,
                    Some((temporary, range)) if range.start == index => {
                        // Workaround because `Temporary` doesn't implement fill/alignment.
                        buffer.clear();
                        write!(&mut buffer, "{}", temporary)?;
                        write!(fmt, "{:1$}", buffer, max_temporary_width)?
                    }
                    Some((_, range)) if range.end == index => {
                        let clobbered = match range.clobbered {
                            Clobbered::Caller => 'C',
                            Clobbered::RaxRdx => 'R',
                            Clobbered::Rdx => 'D',
                            Clobbered::Rax => 'A',
                            Clobbered::None => '●',
                        };
                        write!(fmt, "{:1$}", clobbered, max_temporary_width)?;
                    }
                    Some((_, range)) => {
                        assert!(range.end >= index);
                        assert!(range.start < index);
                        let clobbered = match range.clobbered {
                            Clobbered::None => '|',
                            _ => '┊',
                        };
                        write!(fmt, "{:1$}", clobbered, max_temporary_width)?;
                    }
                }
            }

            let indent = match statement {
                asm::Statement::Label(_) => "",
                _ => "  ",
            };

            writeln!(fmt, " {}{}", indent, statement)?;
        }

        Ok(())
    }
}
