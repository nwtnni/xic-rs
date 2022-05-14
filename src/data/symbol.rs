use std::borrow::Cow;
use std::cell::RefCell;

use crate::Set;

thread_local! {
    /// Global cache of interned strings
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::default());
}

/// Implements naive string internment.
///
/// Requires O(n) heap space to store unique strings, in
/// return for O(1) symbol equality checks and faster symbol hashing.
///
/// Does not garbage collect interned strings: the memory
/// is intentionally leaked for the duration of the program.
#[derive(Debug, Default)]
pub struct Interner(Set<&'static str>);

/// Represents a unique string.
///
/// Only the same `Interner` that produced a `Symbol` can be used
/// to resolve it to a string again.
#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Symbol(usize);

impl Interner {
    /// Store `string` in this interner if not already cached.
    fn intern<'a, S>(&mut self, string: S) -> Symbol
    where
        S: Into<Cow<'a, str>>,
    {
        let cow = string.into();
        if let Some(index) = self.0.get_index_of(cow.as_ref()) {
            Symbol(index)
        } else {
            let owned = cow.into_owned().into_boxed_str();
            let leaked = Box::leak(owned);
            let (index, _) = self.0.insert_full(leaked);
            Symbol(index)
        }
    }

    /// Store static `string` in this interner if not already cached.
    fn intern_static(&mut self, string: &'static str) -> Symbol {
        let (index, _) = self.0.insert_full(string);
        Symbol(index)
    }

    /// Resolve `symbol` in this interner.
    /// Requires that `symbol` was produced by this interner.
    fn resolve(&self, symbol: Symbol) -> &'static str {
        self.0[symbol.0]
    }
}

/// Look up `string` in the global cache, and insert it if missing.
pub fn intern<'a, S>(string: S) -> Symbol
where
    S: Into<Cow<'a, str>>,
{
    INTERNER.with(|interner| interner.borrow_mut().intern(string))
}

/// Look up static `string` in the global cache, and insert it if missing.
pub fn intern_static(string: &'static str) -> Symbol {
    INTERNER.with(|interner| interner.borrow_mut().intern_static(string))
}

/// Resolve `symbol` to its string representation.
pub fn resolve(symbol: Symbol) -> &'static str {
    INTERNER.with(|interner| interner.borrow().resolve(symbol))
}

impl From<Symbol> for &'static str {
    fn from(symbol: Symbol) -> Self {
        resolve(symbol)
    }
}

impl std::str::FromStr for Symbol {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(intern(s))
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", resolve(*self))
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", resolve(*self))
    }
}
