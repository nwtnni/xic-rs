use std::iter;
use std::mem;

use crate::abi::Abi;
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
///
/// There are additional optimization opportunities available to `final` classes,
/// depending on whether or not they have a superclass:
///
/// a) If final class `Class` has no superclass:
///
/// We can completely discard the virtual table:
///
/// - Dispatch all method calls as static function calls
/// - Remove the virtual table pointer from the class layout
/// - Remove the virtual table from the `.bss` section
///
/// This does violate the course Xi++ ABI specification, but only for classes
/// marked `final`, which is itself a language extension. Regular classes will
/// still compile and link against other Xi++ compilers' output.
///
/// b) If final class `Class` has superclass `Superclass`:
///
/// We can't remove the virtual table entirely, as we could have an assignment like:
///
/// ```text
/// super: Superclass = new Class
/// ```
///
/// Here, invoking methods on `super` requires there to be a virtual table.
/// But we can still omit any methods that aren't defined in superclass(es)
/// from the virtual table and dispatch those statically at type `Class`.
pub struct Layout {
    /// First interface-only class in this class's inheritance hierarchy.
    ///
    /// Class size and field offset must be relative to this class's size.
    interface: Option<Symbol>,

    /// Set of fields accessible to this class, ordered by position.
    fields: Set<(Symbol, Symbol)>,

    /// Map from this class's methods to their index in the virtual table.
    methods: Map<Symbol, usize>,

    /// `Some(size)` of this class's virtual table in words, or `None` if
    /// it does not need a virtual table.
    slots: Option<usize>,
}

impl Layout {
    pub fn new(context: &Context, abi: Abi, class: &Symbol) -> Self {
        let mut fields = Set::default();
        let mut methods = Map::default();
        let mut interface = None;
        let mut slots = 0;

        let r#final = match abi {
            Abi::Xi => false,
            Abi::XiFinal => context.get_final(class).is_some(),
        };

        for (superclass, r#final) in context
            .ancestors_inclusive(class)
            .zip(iter::once(Some(r#final)).chain(iter::once(None).cycle()))
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
                use Entry::*;
                match entry {
                    Variable(_) => {
                        fields.insert((superclass, identifier.symbol));
                    }
                    Function(_, _) | Signature(_, _) if matches!(r#final, Some(true)) => continue,
                    Function(_, _) | Signature(_, _) => {
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
            slots: if r#final && slots == 1 {
                None
            } else {
                Some(slots)
            },
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
            .map(|index| index + self.virtual_table_field_offset())
    }

    /// Number of fields relative to superclass, accounting for virtual table pointer
    /// if there is no superclass.
    pub fn field_len(&self) -> usize {
        self.fields.len() + self.virtual_table_field_offset()
    }

    /// The 0th index is reserved for the virtual table pointer, but only if:
    /// a) there is no superclass interface carrying its own virtual table pointer,
    /// b) the virtual table exists.
    fn virtual_table_field_offset(&self) -> usize {
        match (self.interface, self.slots) {
            (Some(_), _) => 0,
            (None, Some(_)) => 1,
            (None, None) => 0,
        }
    }

    /// Index of method in virtual table, accounting for reserved slots per class.
    pub fn method_index(&self, method: &Symbol) -> Option<usize> {
        self.methods.get(method).copied()
    }

    /// Size of virtual table in words, accounting for reserved slots per class.
    pub fn virtual_table_len(&self) -> Option<usize> {
        self.slots
    }
}
