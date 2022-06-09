/// Convenience trait for method-chaining functions.
pub trait Tap: Sized {
    fn tap<F, T>(self, f: F) -> T
    where
        F: FnOnce(Self) -> T,
    {
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
            done: false,
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
            None => None,
            Some(_) if self.done => None,
            Some(next) => {
                self.done = (self.predicate)(&next);
                Some(next)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Or<L, R> {
    L(L),
    R(R),
}

impl<L, R, T> Iterator for Or<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Or::L(left) => left.next(),
            Or::R(right) => right.next(),
        }
    }
}

// Note: the struct definition is inside the macro so that `log::info` will have
// the correct module attached to it (i.e. the module of the macro invocation,
// and not `xic::util`). Seems like there should be a better alternative,
// but this works for now?
macro_rules! time {
    ($($arg:tt)*) => {
        struct Timer {
            start: ::std::time::Instant,
            message: String,
        }

        impl Drop for Timer {
            fn drop(&mut self) {
                let duration = self.start.elapsed();
                log::info!(
                    "{} (took {}.{:03}s)",
                    self.message,
                    duration.as_secs(),
                    duration.subsec_millis()
                );
            }
        }

        let _timer = Timer {
            start: ::std::time::Instant::now(),

            // FIXME: is it possible to avoid heap allocation here?
            // The `format_args` macro returns a temporary with too short a lifetime :(
            message: format!($($arg)*),
        };
    }
}

// https://github.com/rust-lang/rust/pull/52234#issuecomment-976702997
#[doc(hidden)]
pub(crate) use time;
