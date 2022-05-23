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
    /// Set of fields defined in this class, ordered by position.
    fields: Set<Symbol>,

    /// Map from this class's methods to their index in the virtual table.
    methods: Map<Symbol, usize>,

    /// Size of this class's virtual table in words.
    size: usize,
}

impl Layout {
    pub fn new(context: &Context, class: &Symbol) -> Self {
        let mut fields = Set::default();
        let mut methods = Map::default();
        let mut offset = 0;

        #[allow(clippy::explicit_counter_loop)]
        for superclass in context
            .ancestors_inclusive(class)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            // Reserve a slot for private use
            offset += 1;

            let (_, environment) = context
                .get_class(&superclass)
                .expect("[TYPE ERROR]: unbound class");

            for (symbol, (_, entry)) in environment {
                match entry {
                    Entry::Variable(_) => {
                        fields.insert(*symbol);
                    }
                    Entry::Function(_, _) => {
                        methods.entry(*symbol).or_insert_with(|| {
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

        Self {
            fields,
            methods,
            size: offset,
        }
    }

    pub fn field(&self, field: &Symbol) -> Option<usize> {
        self.fields.get_index_of(field)
    }

    pub fn field_len(&self) -> usize {
        self.fields.len()
    }

    pub fn method(&self, method: &Symbol) -> Option<usize> {
        self.methods.get_index_of(method)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
