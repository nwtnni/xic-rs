use std::mem;

use crate::check::Context;
use crate::check::Entry;
use crate::data::symbol::Symbol;
use crate::Map;
use crate::Set;

/// Classes are laid out as follows:
///
/// ```text
///             +-------------------+    +-------------------------+
/// instance -> | virtual table     | -> | superclass private slot |
///             |-------------------|    |-------------------------|
///             | superclass fields |    | superclass methods      |
///             |         .         |    |            .            |
///             |         .         |    |            .            |
///             |         .         |    |            .            |
///             |-------------------|    |-------------------------|
///             | class fields      |    | class private slot      |
///             |         .         |    |-------------------------|
///             |         .         |    | class methods           |
///             |         .         |    |            .            |
///             |         .         |    |            .            |
///             +-------------------+    |            .            |
///                                      +-------------------------+
/// ```
pub struct Layout {
    /// First interface-only class in this class's inheritance hierarchy.
    ///
    /// Class size and field offset must be relative to this class's size.
    interface: Option<Symbol>,

    /// Set of fields accessible to this class, ordered by position.
    fields: Set<(Symbol, Symbol)>,

    /// Map from this class's methods to their index in the virtual table.
    methods: Map<Symbol, usize>,

    /// Size of this class's virtual table in words.
    slots: usize,
}

impl Layout {
    pub fn new(context: &Context, class: &Symbol) -> Self {
        let mut fields = Set::default();
        let mut methods = Map::default();
        let mut interface = None;
        let mut slots = 0;

        for superclass in context
            .ancestors_inclusive(class)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            // Check if last interface before implementation
            //
            // There's a possible edge case here if the superclasses can switch back and
            // forth between implementations and interfaces. All fields after the last
            // interface should be hidden, so this should work.
            if context.get_class_signature(&superclass).is_some()
                && context.get_class_implementation(&superclass).is_none()
            {
                interface = Some(superclass);
            }

            // Reserve a slot for private use
            slots += 1;

            for (identifier, entry) in context
                .get_class(&superclass)
                .expect("[INTERNAL ERROR]: unbound class")
            {
                match entry {
                    Entry::Variable(_) => {
                        fields.insert((superclass, identifier.symbol));
                    }
                    Entry::Function(_, _) | Entry::Signature(_, _) => {
                        methods.entry(identifier.symbol).or_insert_with(|| {
                            let increment = slots + 1;
                            mem::replace(&mut slots, increment)
                        });
                    }
                }
            }
        }

        Self {
            interface,
            fields,
            methods,
            slots,
        }
    }

    /// Name of this class's first ancestor with a signature and no implementation, if it exists.
    ///
    /// Field accesses and class size must be computed relative to this ancestor's class size,
    /// which is unknown at compile time.
    pub fn interface(&self) -> Option<Symbol> {
        self.interface
    }

    /// Index of field relative to superclass, accounting for virtual table pointer
    /// if there is no superclass.
    pub fn field_index(&self, class: &Symbol, field: &Symbol) -> Option<usize> {
        self.fields
            .get_index_of(&(*class, *field))
            // Search for latest field override if no exact match
            .or_else(|| self.fields.iter().rposition(|(_, name)| name == field))
            // 0th index reserved for virtual table pointer, but only if there
            // is no interface carrying its own virtual table pointer
            .map(|index| index + self.interface.map(|_| 0).unwrap_or(1))
    }

    /// Number of fields relative to superclass, accounting for virtual table pointer
    /// if there is no superclass.
    pub fn field_len(&self) -> usize {
        self.fields.len() + self.interface.map(|_| 0).unwrap_or(1)
    }

    /// Index of method in virtual table, accounting for reserved slots per class.
    pub fn method_index(&self, method: &Symbol) -> Option<usize> {
        self.methods.get(method).copied()
    }

    /// Size of virtual table in words, accounting for reserved slots per class.
    pub fn virtual_table_len(&self) -> usize {
        self.slots
    }
}
