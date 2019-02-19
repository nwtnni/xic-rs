use std::collections::HashMap as Map;

use crate::data::typ;
use crate::util::symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Var(typ::Exp),
    Ret(typ::Typ),
    Fun(typ::Typ, typ::Typ),
}

#[derive(Clone, Debug)]
pub struct Env {
    stack: Vec<Map<symbol::Symbol, Entry>>,
}
