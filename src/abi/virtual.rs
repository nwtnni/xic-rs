#![allow(unused)]

use crate::check::Context;
use crate::check::Entry;
use crate::data::symbol::Symbol;
use crate::Map;
use crate::Set;

pub struct Table {
    fields: Map<Symbol, Set<Symbol>>,
    methods: Map<Symbol, Map<Symbol, usize>>,
}

impl Table {
    pub fn new(context: &Context) -> Self {
        let mut fields = Map::default();
        let mut methods = Map::default();

        for class in context.class_implementations() {
            let mut class_fields = Set::default();
            let mut class_methods = Map::default();
            let mut offset = 0;

            #[allow(clippy::explicit_counter_loop)]
            for superclass in context
                .ancestors_inclusive(class)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                offset += 1;

                let (_, environment) = context
                    .get_class(&superclass)
                    .expect("[TYPE ERROR]: unbound class");

                for (symbol, (_, entry)) in environment {
                    match entry {
                        Entry::Variable(_) => {
                            class_fields.insert(*symbol);
                        }
                        Entry::Function(_, _) => {
                            class_methods.entry(*symbol).or_insert_with(|| {
                                let index = offset;
                                offset += 1;
                                index
                            });
                        }
                        Entry::Signature(_, _) => {
                            unreachable!("[TYPE ERROR]: unimplemented signature")
                        }
                    }
                }
            }

            fields.insert(*class, class_fields);
            methods.insert(*class, class_methods);
        }

        Self { fields, methods }
    }

    pub fn field(&self, class: &Symbol, field: &Symbol) -> Option<usize> {
        self.fields.get(class)?.get_index_of(field)
    }

    pub fn field_len(&self, class: &Symbol) -> Option<usize> {
        Some(self.fields.get(class)?.len())
    }

    pub fn method(&self, class: &Symbol, method: &Symbol) -> Option<usize> {
        self.methods.get(class)?.get_index_of(method)
    }

    pub fn method_len(&self, class: &Symbol) -> Option<usize> {
        self.methods
            .get(class)?
            .last()
            .map(|(_, index)| Some(*index + 1))
            .unwrap_or(Some(1))
    }
}
