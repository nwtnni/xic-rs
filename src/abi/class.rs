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
    size: usize,
}

impl Layout {
    pub fn new(context: &Context, class: &Symbol) -> Self {
        let mut fields = Set::default();
        let mut methods = Map::default();
        let mut interface = None;
        let mut offset = 0;

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
            offset += 1;

            let (_, environment) = context
                .get_class(&superclass)
                .expect("[TYPE ERROR]: unbound class");

            for (symbol, (_, entry)) in environment {
                match entry {
                    Entry::Variable(_) => {
                        fields.insert((superclass, *symbol));
                    }
                    Entry::Function(_, _) | Entry::Signature(_, _) => {
                        methods.entry(*symbol).or_insert_with(|| {
                            let index = offset;
                            offset += 1;
                            index
                        });
                    }
                }
            }
        }

        Self {
            interface,
            fields,
            methods,
            size: offset,
        }
    }

    pub fn interface(&self) -> Option<Symbol> {
        self.interface
    }

    pub fn field_index(&self, class: &Symbol, field: &Symbol) -> Option<usize> {
        self.fields
            .get_index_of(&(*class, *field))
            // Search for latest field override if no exact match
            .or_else(|| self.fields.iter().rposition(|(_, name)| name == field))
            // 0th index reserved for virtual table pointer, but only if there
            // is no interface carrying its own virtual table pointer
            .map(|index| index + self.interface.map(|_| 0).unwrap_or(1))
    }

    pub fn field_len(&self) -> usize {
        self.fields.len()
    }

    pub fn method_index(&self, method: &Symbol) -> Option<usize> {
        self.methods.get(method).copied()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
