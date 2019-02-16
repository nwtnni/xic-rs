/// Convenience trait for method-chaining functions.
pub trait Tap: Sized {
    fn tap<F, T>(self, f: F) -> T where F: FnOnce(Self) -> T {
        f(self)
    }
}

impl<T: Sized> Tap for T {}

/// Convenience trait for applying conversions in method chains.
pub trait Conv: Sized {
    fn conv<T: From<Self>>(self) -> T {
        self.into()
    }
}

impl<T: Sized> Conv for T {}

/// Advance the underlying iterator up to and including when the provided
/// predicate returns `true`.
pub trait TakeUntil: Iterator + Sized {
    fn take_until<F: FnMut(&Self::Item) -> bool>(self, predicate: F) -> Until<Self, F> {
        Until {
            inner: self,
            predicate,
            done: false
        }
    }
}

impl<I: Iterator + Sized> TakeUntil for I {}

/// Implementation detail for TakeUntil trait.
pub struct Until<I, F> {
    inner: I,
    predicate: F,
    done: bool,
}

impl<T, I: Iterator<Item = T>, F: FnMut(&T) -> bool> Iterator for Until<I, F> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
        | None => None,
        | Some(_) if self.done => None,
        | Some(next) => {
            self.done = (self.predicate)(&next);
            Some(next)
        }
        }
    }
}

pub fn unescape_char(c: char) -> Option<&'static str> {
    match c {
    | '\n'   => Some("\\n"),
    | '\r'   => Some("\\r"),
    | '\t'   => Some("\\t"),
    | '\x08' => Some("\\b"),
    | '\x0C' => Some("\\f"),
    | _      => None,
    }
}

pub fn unescape_str(s: &str) -> String {
    let mut buffer = String::new();
    for c in s.chars() {
        match unescape_char(c) {
        | Some(s) => buffer.push_str(s),
        | None => buffer.push(c),
        }
    }
    buffer
}

