use std::fmt;
use std::fs;

use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::Map;

/// Represents a single point in a source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub path: Option<Symbol>,
    pub idx: usize,
    pub row: usize,
    pub col: usize,
}

impl Default for Point {
    fn default() -> Self {
        Point {
            path: None,
            idx: 0,
            row: 1,
            col: 1,
        }
    }
}

impl Point {
    /// Constructs the next point in the program.
    /// Assumes that the current character is ASCII.
    pub fn bump(&self) -> Self {
        Point {
            path: self.path,
            idx: self.idx + 1,
            row: self.row,
            col: self.col + 1,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.row, self.col)
    }
}

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
        self.lo.idx
    }

    fn end(&self) -> usize {
        self.hi.idx
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
