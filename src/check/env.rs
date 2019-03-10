use std::collections::HashMap;

use crate::data::typ;
use crate::util::symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Var(typ::Exp),
    Fun(Vec<typ::Exp>, typ::Typ),
    Sig(Vec<typ::Exp>, typ::Typ),
}

#[derive(Clone, Debug)]
pub struct Env {
    stack: Vec<HashMap<symbol::Symbol, Entry>>,
    ret: typ::Typ,
}

impl Env {
    pub fn new() -> Self {
        let library = hashmap! {
            symbol::intern("print") => Entry::Fun(
                vec![typ::Exp::Arr(Box::new(typ::Exp::Int))],
                typ::Typ::Unit,
            ),
            symbol::intern("println") => Entry::Fun(
                vec![typ::Exp::Arr(Box::new(typ::Exp::Int))],
                typ::Typ::Unit,
            ),
            symbol::intern("readln") => Entry::Fun(
                vec![],
                typ::Typ::array(typ::Exp::Int),
            ),
            symbol::intern("getchar") => Entry::Fun(
                vec![],
                typ::Typ::int(),
            ),
            symbol::intern("eof") => Entry::Fun(
                vec![],
                typ::Typ::boolean(),
            ),
            symbol::intern("unparseInt") => Entry::Fun(
                vec![typ::Exp::Int],
                typ::Typ::array(typ::Exp::Int),
            ),
            symbol::intern("parseInt") => Entry::Fun(
                vec![typ::Exp::Arr(Box::new(typ::Exp::Int))],
                typ::Typ::Tup(vec![typ::Exp::Int, typ::Exp::Bool]),
            ),
        };
        Env {
            stack: vec![library],
            ret: typ::Typ::Unit,
        }
    }

    pub fn get(&self, symbol: symbol::Symbol) -> Option<&Entry> {
        for map in self.stack.iter().rev() {
            if let Some(entry) = map.get(&symbol) {
                return Some(entry)
            }
        }
        None
    }

    pub fn insert(&mut self, symbol: symbol::Symbol, entry: Entry) {
        self.stack.last_mut()
            .expect("[INTERNAL ERROR]: missing top-level environment")
            .insert(symbol, entry);
    }

    pub fn remove(&mut self, name: symbol::Symbol) -> Option<Entry> {
        for map in self.stack.iter_mut().rev() {
            if let Some(entry) = map.remove(&name) {
                return Some(entry)
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

    pub fn get_return(&self) -> &typ::Typ {
        &self.ret
    }

    pub fn set_return(&mut self, typ: typ::Typ) {
        self.ret = typ;
    }
}
