#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

impl std::fmt::Display for Point {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}:{}", self.row, self.col)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub lo: Point,
    pub hi: Point,
}

impl std::fmt::Display for Span {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}-{}", self.lo, self.hi)
    }
}

impl From<Point> for Span {
    fn from(point: Point) -> Self {
        Span { lo: point, hi: point }
    }
}
