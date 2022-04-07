use std::collections::HashMap;

use crate::data::r#type;
use crate::util::symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Variable(r#type::Expression),
    Function(Vec<r#type::Expression>, Vec<r#type::Expression>),
    Signature(Vec<r#type::Expression>, Vec<r#type::Expression>),
}

#[derive(Clone, Debug)]
pub struct Context {
    stack: Vec<HashMap<symbol::Symbol, Entry>>,
    r#return: Option<Vec<r#type::Expression>>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            stack: vec![HashMap::default()],
            r#return: None,
        }
    }

    pub fn get(&self, symbol: symbol::Symbol) -> Option<&Entry> {
        for map in self.stack.iter().rev() {
            if let Some(r#type) = map.get(&symbol) {
                return Some(r#type);
            }
        }
        None
    }

    pub fn insert(&mut self, symbol: symbol::Symbol, r#type: Entry) {
        self.stack
            .last_mut()
            .expect("[INTERNAL ERROR]: missing top-level environment")
            .insert(symbol, r#type);
    }

    pub fn remove(&mut self, name: symbol::Symbol) -> Option<Entry> {
        for map in self.stack.iter_mut().rev() {
            if let Some(r#type) = map.remove(&name) {
                return Some(r#type);
            }
        }
        None
    }

    pub fn push(&mut self) {
        self.stack.push(HashMap::default());
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn get_returns(&self) -> &[r#type::Expression] {
        self.r#return
            .as_ref()
            .expect("[INTERNAL ERROR]: missing return type")
    }

    pub fn set_return(&mut self, r#type: Vec<r#type::Expression>) {
        self.r#return.replace(r#type);
    }
}
