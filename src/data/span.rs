use std::fmt;
use std::fs;

use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::Map;

const _: [u8; 12] = [0; std::mem::size_of::<Point>()];

/// Represents a single point in a source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    path: Option<Symbol>,
    index: u32,
    row: u16,
    column: u16,
}

impl Point {
    pub fn new(path: Symbol, index: usize, row: usize, column: usize) -> Self {
        Point {
            path: Some(path),
            index: u32::try_from(index).unwrap(),
            row: u16::try_from(row).unwrap(),
            column: u16::try_from(column).unwrap(),
        }
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }
}

impl Default for Point {
    fn default() -> Self {
        Point {
            path: None,
            index: 0,
            row: 1,
            column: 1,
        }
    }
}

impl Point {
    /// Constructs the next point in the program.
    /// Assumes that the current character is ASCII.
    pub fn bump(&self) -> Self {
        Point {
            path: self.path,
            index: self.index + 1,
            row: self.row,
            column: self.column + 1,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.row, self.column)
    }
}

const _: [u8; 24] = [0; std::mem::size_of::<Span>()];

/// Represents a span of text in a source file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub lo: Point,
    pub hi: Point,
}

impl Span {
    pub fn new(lo: Point, hi: Point) -> Self {
        Span { lo, hi }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.lo)
    }
}

impl From<Point> for Span {
    fn from(point: Point) -> Self {
        Span {
            lo: point,
            hi: point.bump(),
        }
    }
}

impl ariadne::Span for Span {
    type SourceId = Symbol;

    fn source(&self) -> &Self::SourceId {
        self.lo.path.as_ref().or(self.hi.path.as_ref()).unwrap()
    }

    fn start(&self) -> usize {
        self.lo.index as usize
    }

    fn end(&self) -> usize {
        self.hi.index as usize
    }
}

#[derive(Default)]
pub struct FileCache(Map<Symbol, ariadne::Source>);

// https://github.com/zesterer/ariadne/blob/dc1396c5e430b455d3b685db5e773cab689f7c1d/src/source.rs#L140-L159
impl ariadne::Cache<Symbol> for FileCache {
    fn fetch(&mut self, id: &Symbol) -> Result<&ariadne::Source, Box<dyn fmt::Debug + '_>> {
        match self.0.entry(*id) {
            indexmap::map::Entry::Occupied(entry) => Ok(entry.into_mut()),
            indexmap::map::Entry::Vacant(entry) => fs::read_to_string(symbol::resolve(*id))
                .map_err(|error| Box::new(error) as _)
                .map(|source| ariadne::Source::from(&source))
                .map(|source| &*entry.insert(source)),
        }
    }

    fn display<'a>(&self, id: &'a Symbol) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(id))
    }
}
