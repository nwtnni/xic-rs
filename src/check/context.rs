use crate::data::r#type;
use crate::Map;

use crate::data::symbol::Symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Variable(r#type::Expression),
    Function(Vec<r#type::Expression>, Vec<r#type::Expression>),
    Signature(Vec<r#type::Expression>, Vec<r#type::Expression>),
}

#[derive(Clone, Debug)]
pub struct Context {
    /// Class hierarchy mapping subtype to supertype
    hierarchy: Map<Symbol, Symbol>,

    /// Globally-scoped global variables and functions
    globals: Map<Symbol, Entry>,

    /// Class-scoped method and fields
    classes: Map<Symbol, Map<Symbol, Entry>>,

    /// Locally scoped variables
    locals: Vec<(LocalScope, Map<Symbol, Entry>)>,
}

#[derive(Copy, Clone, Debug)]
pub enum Scope {
    Global(GlobalScope),
    Local,
}

impl From<GlobalScope> for Scope {
    fn from(scope: GlobalScope) -> Self {
        Self::Global(scope)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GlobalScope {
    Global,
    Class(Symbol),
}

#[derive(Clone, Debug)]
pub enum LocalScope {
    Method {
        class: Symbol,
        returns: Vec<r#type::Expression>,
    },
    Function {
        returns: Vec<r#type::Expression>,
    },
    Block,
    If,
    Else,
    While,
}

impl LocalScope {
    fn returns(&self) -> Option<&[r#type::Expression]> {
        match self {
            LocalScope::Block | LocalScope::If | LocalScope::Else | LocalScope::While => None,
            LocalScope::Method { class: _, returns } | LocalScope::Function { returns } => {
                Some(returns.as_slice())
            }
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            globals: Map::default(),
            classes: Map::default(),
            locals: Vec::default(),
            hierarchy: Map::default(),
        }
    }

    pub fn get<S: Into<Scope>>(&self, scope: S, symbol: &Symbol) -> Option<&Entry> {
        match scope.into() {
            Scope::Global(GlobalScope::Global) => self.globals.get(symbol),
            Scope::Global(GlobalScope::Class(class)) => {
                self.classes.get(&class).and_then(|class| class.get(symbol))
            }
            Scope::Local => self
                .locals
                .iter()
                .rev()
                .find_map(|(scope, r#types)| {
                    r#types
                        .get(symbol)
                        .or_else(|| self.get_class(scope, symbol))
                })
                .or_else(|| self.globals.get(symbol)),
        }
    }

    fn get_class(&self, scope: &LocalScope, symbol: &Symbol) -> Option<&Entry> {
        let mut class = match scope {
            LocalScope::Method { class, returns: _ } => class,
            LocalScope::Function { returns: _ }
            | LocalScope::Block
            | LocalScope::If
            | LocalScope::Else
            | LocalScope::While => return None,
        };

        loop {
            if let Some(entry) = self.classes[class].get(symbol) {
                return Some(entry);
            }

            class = self.hierarchy.get(class)?;
        }
    }

    pub fn insert<S: Into<Scope>>(
        &mut self,
        scope: S,
        symbol: Symbol,
        r#type: Entry,
    ) -> Option<Entry> {
        match scope.into() {
            Scope::Global(GlobalScope::Global) => self.globals.insert(symbol, r#type),
            Scope::Global(GlobalScope::Class(class)) => self
                .classes
                .entry(class)
                .or_default()
                .insert(symbol, r#type),
            Scope::Local => self
                .locals
                .last_mut()
                .expect("[INTERNAL ERROR]: missing environment")
                .1
                .insert(symbol, r#type),
        }
    }

    pub fn insert_subtype(&mut self, subtype: Symbol, supertype: Symbol) -> Option<Symbol> {
        self.hierarchy
            .insert(subtype, supertype)
            .filter(|existing| *existing != supertype)
    }

    pub fn push(&mut self, scope: LocalScope) {
        self.locals.push((scope, Map::default()));
    }

    pub fn pop(&mut self) {
        self.locals.pop();
    }

    pub fn get_returns(&self) -> Option<&[r#type::Expression]> {
        self.locals
            .iter()
            .rev()
            .find_map(|(scope, _)| scope.returns())
    }

    pub fn is_subtype(&self, subtype: &r#type::Expression, supertype: &r#type::Expression) -> bool {
        use r#type::Expression::*;
        match (subtype, supertype) {
            (Any, _) | (Integer, Integer) | (Boolean, Boolean) => true,
            (Array(subtype), Array(supertype)) => self.is_subtype(subtype, supertype),
            (Class(subtype), Class(supertype)) if subtype == supertype => true,
            (Class(mut subtype), Class(supertype)) => loop {
                match self.hierarchy.get(&subtype) {
                    None => return false,
                    Some(r#type) if r#type == supertype => return true,
                    Some(r#type) => subtype = *r#type,
                }
            },
            (_, _) => false,
        }
    }

    pub fn all_subtype<'l, 'r, L, R>(&self, subtypes: L, supertypes: R) -> bool
    where
        L: IntoIterator<Item = &'l r#type::Expression>,
        R: IntoIterator<Item = &'r r#type::Expression>,
    {
        let mut subtypes = subtypes.into_iter();
        let mut supertypes = supertypes.into_iter();
        loop {
            match (subtypes.next(), supertypes.next()) {
                (None, None) => return true,
                (None, Some(_)) | (Some(_), None) => return false,
                (Some(subtype), Some(supertype)) if self.is_subtype(subtype, supertype) => (),
                (Some(_), Some(_)) => return false,
            }
        }
    }

    pub fn least_upper_bound(
        &self,
        left: &r#type::Expression,
        right: &r#type::Expression,
    ) -> Option<r#type::Expression> {
        use r#type::Expression::*;
        match (left, right) {
            (Any, r#type) | (r#type, Any) => Some(r#type.clone()),
            (Integer, Integer) => Some(Integer),
            (Boolean, Boolean) => Some(Boolean),
            (Array(left), Array(right)) => {
                self.least_upper_bound(left, right).map(Box::new).map(Array)
            }
            (Class(_), Class(_)) => {
                if self.is_subtype(left, right) {
                    Some(right.clone())
                } else if self.is_subtype(right, left) {
                    Some(left.clone())
                } else {
                    None
                }
            }
            (_, _) => None,
        }
    }
}
