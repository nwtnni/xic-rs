#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub idx: usize,
    pub row: usize,
    pub col: usize,
}

impl Point {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        write!(fmt, "{}-{}", self.lo, self.hi)
    }
}

impl From<Point> for Span {
    fn from(point: Point) -> Self {
        Span { lo: point, hi: point.bump() }
    }
}
