pub trait Tap: Sized {
    fn tap<F, T>(self, f: F) -> T where F: FnOnce(Self) -> T {
        f(self)
    }
}

impl<T: Sized> Tap for T {}
