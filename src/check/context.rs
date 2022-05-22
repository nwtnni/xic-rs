use std::iter;

use crate::data::ast::Identifier;
use crate::data::r#type;
use crate::data::span::Span;
use crate::data::symbol::Symbol;
use crate::Map;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Variable(r#type::Expression),
    Function(Vec<r#type::Expression>, Vec<r#type::Expression>),
    Signature(Vec<r#type::Expression>, Vec<r#type::Expression>),
}

type Environment = Map<Symbol, (Span, Entry)>;

#[derive(Clone, Debug)]
pub struct Context {
    /// Class hierarchy mapping subtype to supertype
    hierarchy: Map<Symbol, Identifier>,

    /// Globally-scoped global variables and functions
    globals: Environment,

    /// Class-scoped method and fields
    classes: Map<Symbol, (Span, Environment)>,

    /// Set of classes declared in interfaces
    class_signatures: Map<Symbol, Span>,

    /// Set of classes in implementation module
    class_implementations: Map<Symbol, Span>,

    /// Locally scoped variables
    locals: Vec<(LocalScope, Environment)>,
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
            class_signatures: Map::default(),
            class_implementations: Map::default(),
            locals: Vec::default(),
            hierarchy: Map::default(),
        }
    }

    pub fn get<S: Into<Scope>>(&self, scope: S, symbol: &Symbol) -> Option<&Entry> {
        self.get_full(scope, symbol).map(|(_, entry)| entry)
    }

    pub fn get_full<S: Into<Scope>>(&self, scope: S, symbol: &Symbol) -> Option<&(Span, Entry)> {
        match scope.into() {
            Scope::Global(GlobalScope::Global) => self.globals.get(symbol),
            Scope::Global(GlobalScope::Class(class)) => self
                .ancestors_inclusive(&class)
                .map(|class| &self.classes[&class])
                .find_map(|(_, class)| class.get(symbol)),
            Scope::Local => self
                .locals
                .iter()
                .rev()
                .find_map(|(_, r#types)| r#types.get(symbol))
                .or_else(|| self.get_full(GlobalScope::Global, symbol))
                .or_else(|| {
                    let class = self.get_scoped_class()?;
                    self.get_full(GlobalScope::Class(class), symbol)
                }),
        }
    }

    pub fn insert_full<S: Into<Scope>>(
        &mut self,
        scope: S,
        identifier: &Identifier,
        r#type: Entry,
    ) -> Option<(Span, Entry)> {
        match scope.into() {
            Scope::Global(GlobalScope::Global) => self
                .globals
                .insert(identifier.symbol, (*identifier.span, r#type)),
            Scope::Global(GlobalScope::Class(class)) => self.classes[&class]
                .1
                .insert(identifier.symbol, (*identifier.span, r#type)),
            Scope::Local => self
                .locals
                .last_mut()
                .expect("[INTERNAL ERROR]: missing environment")
                .1
                .insert(identifier.symbol, (*identifier.span, r#type))
                // Note: inserting into local scope conflicts with names in the global
                // scope, but does *not* conflict with names in the class scope, which
                // can be disambiguated by the type of the receiver.
                .or_else(|| {
                    self.locals
                        .iter_mut()
                        .rev()
                        .skip(1)
                        .find_map(|(_, types)| types.get(&identifier.symbol).cloned())
                        .or_else(|| {
                            self.get_full(GlobalScope::Global, &identifier.symbol)
                                .cloned()
                        })
                }),
        }
    }

    pub fn get_class_signature(&self, class: &Symbol) -> Option<&Span> {
        self.class_signatures.get(class)
    }

    pub fn insert_class_signature(&mut self, class: &Identifier) -> Option<(Span, Environment)> {
        self.class_signatures.insert(class.symbol, *class.span);
        self.classes
            .insert(class.symbol, (*class.span, Map::default()))
    }

    pub fn class_implementations(&self) -> impl Iterator<Item = &Symbol> + '_ {
        self.class_implementations.keys()
    }

    pub fn get_class_implementation(&self, class: &Symbol) -> Option<&Span> {
        self.class_implementations.get(class)
    }

    pub fn insert_class_implementation(&mut self, class: &Identifier) -> Option<Span> {
        if let Some(span) = self.class_implementations.insert(class.symbol, *class.span) {
            return Some(span);
        }

        self.classes
            .entry(class.symbol)
            .and_modify(|(span, _)| *span = *class.span)
            .or_insert_with(|| (*class.span, Map::default()));

        None
    }

    pub fn insert_supertype(
        &mut self,
        subtype: Identifier,
        supertype: Identifier,
    ) -> Option<Identifier> {
        let expected = supertype.symbol;
        self.hierarchy
            .insert(subtype.symbol, supertype)
            .filter(|actual| actual.symbol != expected)
    }

    pub fn has_cycle(&self, subtype: &Identifier) -> bool {
        self.ancestors_exclusive(&subtype.symbol)
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
            let supertype = self
                .hierarchy
                .get(&r#type)
                .map(|identifier| identifier.symbol)?;
            r#type = supertype;
            Some(supertype)
        })
    }

    pub fn get_class(&self, class: &Symbol) -> Option<&(Span, Environment)> {
        self.classes.get(class)
    }

    pub fn get_superclass(&self, class: &Symbol) -> Option<Symbol> {
        self.hierarchy
            .get(class)
            .map(|identifier| identifier.symbol)
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
            (Class(subtype), Class(supertype)) => self
                .ancestors_exclusive(subtype)
                .any(|r#type| r#type == *supertype),
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
