use std::iter;

use crate::data::ast::Identifier;
use crate::data::r#type;
use crate::data::symbol::Symbol;
use crate::Map;

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

#[derive(Clone, Debug)]
pub(super) enum LeastUpperBound {
    Left(r#type::Expression),
    Right(r#type::Expression),
}

impl LeastUpperBound {
    fn array(self) -> Self {
        match self {
            LeastUpperBound::Left(r#type) => {
                LeastUpperBound::Left(r#type::Expression::Array(Box::new(r#type)))
            }
            LeastUpperBound::Right(r#type) => {
                LeastUpperBound::Right(r#type::Expression::Array(Box::new(r#type)))
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
            Scope::Global(GlobalScope::Class(class)) => self
                .ancestors_inclusive(&class)
                .map(|class| &self.classes[&class])
                .find_map(|class| class.get(symbol)),
            Scope::Local => self
                .locals
                .iter()
                .rev()
                .find_map(|(_, r#types)| r#types.get(symbol))
                .or_else(|| self.get(GlobalScope::Global, symbol))
                .or_else(|| {
                    let class = self.get_scoped_class()?;
                    self.get(GlobalScope::Class(class), symbol)
                }),
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
            Scope::Global(GlobalScope::Class(class)) => self.classes[&class].insert(symbol, r#type),
            Scope::Local => self
                .locals
                .last_mut()
                .expect("[INTERNAL ERROR]: missing environment")
                .1
                .insert(symbol, r#type)
                // Note: inserting into local scope conflicts with names in the global
                // scope, but does *not* conflict with names in the class scope, which
                // can be disambiguated by the type of the receiver.
                .or_else(|| {
                    self.locals
                        .iter_mut()
                        .rev()
                        .skip(1)
                        .find_map(|(_, types)| types.get(&symbol).cloned())
                        .or_else(|| self.get(GlobalScope::Global, &symbol).cloned())
                }),
        }
    }

    pub fn insert_class(&mut self, class: Identifier) -> Option<Map<Symbol, Entry>> {
        self.classes.insert(class.symbol, Map::default())
    }

    pub fn get_class(&self, class: &Symbol) -> Option<&Map<Symbol, Entry>> {
        self.classes.get(class)
    }

    pub fn insert_supertype(
        &mut self,
        subtype: Identifier,
        supertype: Identifier,
    ) -> Option<Symbol> {
        self.hierarchy
            .insert(subtype.symbol, supertype.symbol)
            .filter(|existing| *existing != supertype.symbol)
    }

    pub fn has_cycle(&self, subtype: &Identifier) -> bool {
        self.ancestors_exclusive(subtype)
            .any(|supertype| subtype.symbol == supertype)
    }

    pub fn push(&mut self, scope: LocalScope) {
        self.locals.push((scope, Map::default()));
    }

    pub fn pop(&mut self) {
        self.locals.pop();
    }

    pub fn ancestors_inclusive(&self, class: &Symbol) -> impl Iterator<Item = Symbol> + '_ {
        iter::once(*class).chain(self.ancestors_exclusive(class))
    }

    pub fn ancestors_exclusive(&self, class: &Symbol) -> impl Iterator<Item = Symbol> + '_ {
        let mut r#type = *class;
        iter::from_fn(move || {
            let supertype = self.hierarchy.get(&r#type).copied()?;
            r#type = supertype;
            Some(supertype)
        })
    }

    pub fn get_scoped_class(&self) -> Option<Symbol> {
        match self.locals.first()? {
            (LocalScope::Method { class, returns: _ }, _) => Some(*class),
            (LocalScope::Function { returns: _ }, _) => None,
            _ => unreachable!(),
        }
    }

    pub fn get_scoped_returns(&self) -> Option<&[r#type::Expression]> {
        match self.locals.first()? {
            (LocalScope::Method { class: _, returns } | LocalScope::Function { returns }, _) => {
                Some(returns.as_slice())
            }
            _ => unreachable!(),
        }
    }

    pub fn is_subtype(&self, subtype: &r#type::Expression, supertype: &r#type::Expression) -> bool {
        use r#type::Expression::*;
        match (subtype, supertype) {
            (Any, _) | (Integer, Integer) | (Boolean, Boolean) => true,
            (Array(subtype), Array(supertype)) => self.is_subtype_array(subtype, supertype),
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

    fn is_subtype_array(
        &self,
        subtype: &r#type::Expression,
        supertype: &r#type::Expression,
    ) -> bool {
        use r#type::Expression::*;
        match (subtype, supertype) {
            (Any, _) => true,
            (Array(subtype), Array(supertype)) => self.is_subtype_array(subtype, supertype),
            (_, _) => subtype == supertype,
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

    pub(super) fn least_upper_bound(
        &self,
        left: &r#type::Expression,
        right: &r#type::Expression,
    ) -> Option<LeastUpperBound> {
        use r#type::Expression::*;
        match (left, right) {
            (r#type, Any) => Some(LeastUpperBound::Left(r#type.clone())),
            (Any, r#type) => Some(LeastUpperBound::Right(r#type.clone())),
            (Integer, Integer) => Some(LeastUpperBound::Left(Integer)),
            (Boolean, Boolean) => Some(LeastUpperBound::Left(Boolean)),
            (Array(left), Array(right)) => self
                .least_upper_bound_array(left, right)
                .map(LeastUpperBound::array),
            (Class(_), Class(_)) => {
                if self.is_subtype(right, left) {
                    Some(LeastUpperBound::Left(left.clone()))
                } else if self.is_subtype(left, right) {
                    Some(LeastUpperBound::Right(right.clone()))
                } else {
                    None
                }
            }
            (_, _) => None,
        }
    }

    fn least_upper_bound_array(
        &self,
        left: &r#type::Expression,
        right: &r#type::Expression,
    ) -> Option<LeastUpperBound> {
        use r#type::Expression::*;
        match (left, right) {
            (r#type, Any) => Some(LeastUpperBound::Left(r#type.clone())),
            (Any, r#type) => Some(LeastUpperBound::Right(r#type.clone())),
            (Array(left), Array(right)) => self
                .least_upper_bound(left, right)
                .map(LeastUpperBound::array),
            (_, _) if left == right => Some(LeastUpperBound::Left(left.clone())),
            (_, _) => None,
        }
    }
}
