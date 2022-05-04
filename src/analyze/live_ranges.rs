use std::cmp;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;

use crate::abi;
use crate::analyze::analyze;
use crate::analyze::Analysis as _;
use crate::analyze::LiveVariables;
use crate::cfg;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::asm::Assembly;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
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

    /// Track whether this range crosses a function call, and therefore
    /// whether or not it can use caller-saved registers.
    pub clobbered: bool,
}

impl Range {
    fn new(index: usize) -> Self {
        Range {
            start: index,
            end: index,
            clobbered: false,
        }
    }
}

impl LiveRanges {
    pub fn new(cfg: &Cfg<asm::Function<Temporary>>) -> Self {
        let (analysis, solution) = analyze::<LiveVariables<_>, _>(cfg);
        let function = cfg::destruct_cfg(cfg);

        // Walk backward through straight-line abstract assembly to find basic blocks
        let labels =
            function
                .instructions
                .iter()
                .enumerate()
                .rev()
                .filter_map(|(index, instruction)| match instruction {
                    Assembly::Label(label) => Some((index, *label)),
                    _ => None,
                });

        let mut ranges = BTreeMap::<Temporary, Range>::new();

        // Index of the instruction we're transfering over.
        //
        // Careful: this starts at +1 to capture the dataflow input at the end
        // of each block, before starting to transfer backwards for each instruction.
        //
        // With this convention, a range's start is the instruction *after* it is
        // first defined, and a range's end is the instruction where it is first used.
        let mut index = function.instructions.len();

        for (start, label) in labels {
            let mut output = solution.inputs[&label].clone();

            for temporary in &output {
                ranges
                    .entry(*temporary)
                    .and_modify(|range| range.start = index)
                    .or_insert_with(|| Range::new(index));
            }

            while index > start {
                index -= 1;

                let instruction = &function.instructions[index];

                analysis.transfer(instruction, &mut output);

                let clobbered = match instruction {
                    // Allow caller-saved registers to be used across _xi_out_of_bounds
                    // calls, because it diverges anyway.
                    Assembly::Unary(
                        asm::Unary::Call { .. },
                        operand::Unary::I(Immediate::Label(Label::Fixed(label))),
                    ) => symbol::resolve(*label) != abi::XI_OUT_OF_BOUNDS,
                    Assembly::Unary(asm::Unary::Call { .. }, _) => true,
                    _ => false,
                };

                for temporary in &output {
                    ranges
                        .entry(*temporary)
                        .and_modify(|range| {
                            range.clobbered |= clobbered;
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

        for (index, instruction) in self.function.instructions.iter().enumerate() {
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
                    Some((_, range)) if range.end == index => write!(
                        fmt,
                        "{:1$}",
                        if range.clobbered { '◌' } else { '●' },
                        max_temporary_width
                    )?,
                    Some((_, range)) => {
                        assert!(range.end >= index);
                        assert!(range.start < index);
                        write!(
                            fmt,
                            "{:1$}",
                            if range.clobbered { '┊' } else { '|' },
                            max_temporary_width,
                        )?;
                    }
                }
            }

            writeln!(fmt, " {}", instruction)?;
        }

        Ok(())
    }
}
