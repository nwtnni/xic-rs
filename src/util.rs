pub trait Tap: Sized {
    fn tap<F, T>(self, f: F) -> T where F: FnOnce(Self) -> T {
        f(self)
    }
}

impl<T: Sized> Tap for T {}

pub trait Conv: Sized {
    fn conv<T: From<Self>>(self) -> T {
        self.into()
    }
}

impl<T: Sized> Conv for T {}

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