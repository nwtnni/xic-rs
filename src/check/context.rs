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
    scopes: Vec<(Scope, Map<Symbol, Entry>)>,
}

#[derive(Clone, Debug)]
pub enum Scope {
    Global,
    Function { returns: Vec<r#type::Expression> },
    Block,
    If,
    Else,
    While,
}

impl Scope {
    fn returns(&self) -> Option<&[r#type::Expression]> {
        match self {
            Scope::Global | Scope::Block | Scope::If | Scope::Else | Scope::While => None,
            Scope::Function { returns } => Some(returns.as_slice()),
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
            scopes: vec![(Scope::Global, Map::default())],
        }
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&Entry> {
        self.scopes
            .iter()
            .rev()
            .find_map(|(_, r#types)| r#types.get(symbol))
    }

    pub fn insert(&mut self, symbol: Symbol, r#type: Entry) {
        self.scopes
            .last_mut()
            .expect("[INTERNAL ERROR]: missing global environment")
            .1
            .insert(symbol, r#type);
    }

    pub fn remove(&mut self, symbol: &Symbol) -> Option<Entry> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|(_, r#types)| r#types.remove(symbol))
    }

    pub fn push(&mut self, scope: Scope) {
        self.scopes.push((scope, Map::default()));
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn get_returns(&self) -> Option<&[r#type::Expression]> {
        self.scopes
            .iter()
            .rev()
            .find_map(|(scope, _)| scope.returns())
    }
}
