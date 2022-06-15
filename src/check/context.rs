use std::iter;
use std::ops;

use crate::data::ast;
use crate::data::ast::Identifier;
use crate::data::operand::Label;
use crate::data::r#type;
use crate::data::span::Span;
use crate::data::symbol::Symbol;
use crate::Map;
use crate::Set;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Variable(r#type::Expression),
    Function(Vec<r#type::Expression>, Vec<r#type::Expression>),
    Signature(Vec<r#type::Expression>, Vec<r#type::Expression>),
}

#[derive(Clone, Debug)]
pub struct Context {
    /// Class hierarchy mapping subtype to supertype
    hierarchy: Map<Symbol, Identifier>,

    /// Globally-scoped global variables and functions
    globals: Environment<Entry>,

    /// Class-scoped method and fields
    classes: Environment<Environment<Entry>>,

    /// Set of classes declared in interfaces
    class_signatures: Set<Identifier>,

    /// Set of classes in implementation module
    class_implementations: Set<Identifier>,

    /// Set of class templates visible to program
    class_templates: Environment<ast::ClassTemplate>,

    /// Set of function templates visible to program
    function_templates: Environment<ast::FunctionTemplate>,

    /// Locally scoped variables
    locals: Vec<(LocalScope, Environment<Entry>)>,
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
    While(Option<Label>),
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
            globals: Environment::default(),
            classes: Environment::default(),
            class_signatures: Set::default(),
            class_implementations: Set::default(),
            class_templates: Environment::default(),
            function_templates: Environment::default(),
            locals: Vec::default(),
            hierarchy: Map::default(),
        }
    }

    pub fn get<S, K>(&self, scope: S, identifier: &K) -> Option<&Entry>
    where
        S: Into<Scope>,
        K: Key,
    {
        self.get_full(scope, identifier).map(|(_, entry)| entry)
    }

    pub fn get_full<S, K>(&self, scope: S, identifier: &K) -> Option<(&Span, &Entry)>
    where
        S: Into<Scope>,
        K: Key,
    {
        match scope.into() {
            Scope::Global(GlobalScope::Global) => self.globals.get(identifier),
            Scope::Global(GlobalScope::Class(class)) => self
                .ancestors_inclusive(&class)
                .map(|class| &self.classes[class])
                .find_map(|class| class.get(identifier)),
            Scope::Local => self
                .locals
                .iter()
                .rev()
                .find_map(|(_, r#types)| r#types.get(identifier))
                .or_else(|| self.get_full(GlobalScope::Global, identifier))
                .or_else(|| {
                    let class = self.get_scoped_class()?;
                    self.get_full(GlobalScope::Class(class), identifier)
                }),
        }
    }

    pub fn insert<S: Into<Scope>>(
        &mut self,
        scope: S,
        identifier: Identifier,
        r#type: Entry,
    ) -> Option<(Span, Entry)> {
        match scope.into() {
            Scope::Global(GlobalScope::Global) => self.globals.insert(identifier, r#type),
            Scope::Global(GlobalScope::Class(class)) => {
                self.classes[class].insert(identifier, r#type)
            }
            Scope::Local => self
                .locals
                .last_mut()
                .expect("[INTERNAL ERROR]: missing environment")
                .1
                .insert(identifier.clone(), r#type)
                .or_else(|| {
                    self.locals.iter_mut().rev().skip(1).find_map(|(_, types)| {
                        types
                            .get(&identifier)
                            .map(|(span, entry)| (*span, entry.clone()))
                    })
                }),
        }
    }

    pub fn class_signatures(&self) -> impl Iterator<Item = &Symbol> + '_ {
        self.class_signatures.iter().map(|class| &class.symbol)
    }

    pub fn get_class_signature<K: Key>(&self, class: &K) -> Option<&Span> {
        self.class_signatures.get(class).map(|class| &*class.span)
    }

    pub fn insert_class_signature(
        &mut self,
        class: Identifier,
    ) -> Option<(Span, Environment<Entry>)> {
        self.class_signatures.insert(class.clone());
        self.classes.insert(class, Environment::default())
    }

    pub fn class_implementations(&self) -> impl Iterator<Item = &Symbol> + '_ {
        self.class_implementations.iter().map(|class| &class.symbol)
    }

    pub fn get_class_implementation<K: Key>(&self, class: &K) -> Option<&Span> {
        self.class_implementations
            .get(class)
            .map(|class| &*class.span)
    }

    pub fn insert_class_implementation(&mut self, class: Identifier) -> Option<Span> {
        match self.class_implementations.insert_full(class.clone()) {
            (index, false) => return Some(*self.class_implementations[index].span),
            (_, true) => (),
        }

        self.classes.initialize(class);
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

    // TODO: unify template namespace with rest of global namespace?
    pub fn insert_class_template(
        &mut self,
        class: ast::ClassTemplate,
    ) -> Option<(Span, ast::ClassTemplate)> {
        self.class_templates.insert(class.name.clone(), class)
    }

    pub fn get_class_template<K: Key>(&self, identifier: &K) -> Option<&ast::ClassTemplate> {
        self.class_templates.get(identifier).map(|(_, class)| class)
    }

    pub fn insert_function_template(
        &mut self,
        function: ast::FunctionTemplate,
    ) -> Option<(Span, ast::FunctionTemplate)> {
        self.function_templates
            .insert(function.name.clone(), function)
    }

    pub fn get_function_template<K: Key>(&self, identifier: &K) -> Option<&ast::FunctionTemplate> {
        self.function_templates
            .get(identifier)
            .map(|(_, function)| function)
    }

    pub fn push(&mut self, scope: LocalScope) {
        self.locals.push((scope, Environment::default()));
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

    pub fn get_class<K: Key>(&self, class: &K) -> Option<&Environment<Entry>> {
        self.classes.get(class).map(|(_, environment)| environment)
    }

    pub fn get_class_full<K: Key>(&self, class: &K) -> Option<(&Span, &Environment<Entry>)> {
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

    pub fn get_scoped_while(&self) -> Option<Option<Label>> {
        self.locals
            .iter()
            .rev()
            .find_map(|scope| match scope {
                (LocalScope::While(label), _) => Some(label),
                _ => None,
            })
            .cloned()
    }

    pub fn is_subtype(&self, subtype: &r#type::Expression, supertype: &r#type::Expression) -> bool {
        use r#type::Expression::*;
        match (subtype, supertype) {
            (Any, _) | (Null, Class(_)) => true,
            (Array(subtype), Array(supertype)) => self.is_subtype_array(subtype, supertype),
            (Class(subtype), Class(supertype)) if subtype == supertype => true,
            (Class(subtype), Class(supertype)) => self
                .ancestors_exclusive(subtype)
                .any(|r#type| r#type == *supertype),
            (_, _) => subtype == supertype,
        }
    }

    fn is_subtype_array(
        &self,
        subtype: &r#type::Expression,
        supertype: &r#type::Expression,
    ) -> bool {
        use r#type::Expression::*;
        match (subtype, supertype) {
            (Any, _) | (Null, Class(_)) => true,
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
            (r#type @ Class(_), Null) => Some(LeastUpperBound::Left(r#type.clone())),
            (Null, r#type @ Class(_)) => Some(LeastUpperBound::Right(r#type.clone())),
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
            (left, right) if left == right => Some(LeastUpperBound::Left(left.clone())),
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
            (r#type @ Class(_), Null) => Some(LeastUpperBound::Left(r#type.clone())),
            (Null, r#type @ Class(_)) => Some(LeastUpperBound::Right(r#type.clone())),
            (Array(left), Array(right)) => self
                .least_upper_bound(left, right)
                .map(LeastUpperBound::array),
            (_, _) if left == right => Some(LeastUpperBound::Left(left.clone())),
            (_, _) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Environment<T>(Map<Identifier, T>);

impl<T> Default for Environment<T> {
    fn default() -> Self {
        Self(Map::default())
    }
}

impl<T: Clone> Environment<T> {
    pub fn get<K: Key>(&self, key: &K) -> Option<(&Span, &T)> {
        let (identifier, entry) = self.0.get_key_value(key)?;
        Some((&*identifier.span, entry))
    }

    fn insert(&mut self, identifier: Identifier, entry: T) -> Option<(Span, T)> {
        match self.0.swap_remove_full(&identifier) {
            Some((old_index, old_identifier, environment)) => {
                let (new_index, _) = self.0.insert_full(identifier, entry);
                self.0.swap_indices(old_index, new_index);
                Some((*old_identifier.span, environment))
            }
            None => {
                self.0.insert(identifier, entry);
                None
            }
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> indexmap::map::Iter<Identifier, T> {
        self.0.iter()
    }
}

impl<T: Default> Environment<T> {
    fn initialize(&mut self, identifier: Identifier) {
        match self.0.swap_remove_full(&identifier) {
            Some((old, _, environment)) => {
                let (new, _) = self.0.insert_full(identifier, environment);
                self.0.swap_indices(old, new);
            }
            None => {
                self.0.insert(identifier, T::default());
            }
        }
    }
}

impl indexmap::Equivalent<Identifier> for Symbol {
    fn equivalent(&self, key: &Identifier) -> bool {
        *self == key.symbol
    }
}

impl<T> ops::Index<Symbol> for Environment<T> {
    type Output = T;
    fn index(&self, index: Symbol) -> &Self::Output {
        &self.0[&index]
    }
}

impl<T> ops::IndexMut<Symbol> for Environment<T> {
    fn index_mut(&mut self, index: Symbol) -> &mut Self::Output {
        &mut self.0[&index]
    }
}

impl<'a, T> IntoIterator for &'a Environment<T> {
    type Item = (&'a Identifier, &'a T);
    type IntoIter = indexmap::map::Iter<'a, Identifier, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub trait Key: indexmap::Equivalent<Identifier> + std::hash::Hash {}

impl<T> Key for T where T: indexmap::Equivalent<Identifier> + std::hash::Hash {}
