/// Represents a single point in a source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub idx: usize,
    pub row: usize,
    pub col: usize,
}

impl Default for Point {
    fn default() -> Self {
        Point {
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
            idx: self.idx + 1,
            row: self.row,
            col: self.col + 1,
        }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
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

impl std::fmt::Display for Span {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
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
