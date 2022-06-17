use crate::data::ast;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::symbol::Symbol;
use crate::Map;

#[derive(Clone, Debug)]
pub struct Unit<T> {
    pub name: Symbol,
    pub functions: Map<Symbol, T>,
    pub data: Map<Label, Vec<Immediate>>,
    pub bss: Map<Symbol, (Linkage, usize)>,
}

impl<T> Unit<T> {
    pub fn map<F: FnMut(T) -> U, U>(self, mut apply: F) -> Unit<U> {
        Unit {
            name: self.name,
            functions: self
                .functions
                .into_iter()
                .map(|(symbol, function)| (symbol, apply(function)))
                .collect(),
            data: self.data,
            bss: self.bss,
        }
    }

    pub fn map_ref<'a, F: FnMut(&'a T) -> U, U>(&'a self, mut apply: F) -> Unit<U> {
        Unit {
            name: self.name,
            functions: self
                .functions
                .iter()
                .map(|(symbol, function)| (*symbol, apply(function)))
                .collect(),
            data: self.data.clone(),
            bss: self.bss.clone(),
        }
    }

    pub fn map_mut<F: FnMut(&mut T)>(mut self, mut apply: F) -> Self {
        self.functions
            .iter_mut()
            .for_each(|(_, function)| apply(function));
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Linkage {
    /// Locally scoped symbol
    ///
    /// - global: n
    /// - inline: y
    /// - discard: y
    /// - merge: n
    Local,

    /// Globally unique symbol
    ///
    /// - global: y
    /// - inline: y
    /// - discard: n
    /// - merge: n
    Global,

    /// See https://llvm.org/docs/LangRef.html#linkage-types
    ///
    /// Used for template instantiations and utility functions like `_xi_memdup`.
    ///
    /// - global: y
    /// - inline: y
    /// - discard: y
    /// - merge: y
    LinkOnceOdr,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Binary {
    Add,
    Sub,
    Mul,
    Hul,
    Div,
    Mod,
    Xor,
    And,
    Or,
}

impl From<ast::Binary> for Binary {
    fn from(binary: ast::Binary) -> Self {
        match binary {
            ast::Binary::Mul => Binary::Mul,
            ast::Binary::Hul => Binary::Hul,
            ast::Binary::Div => Binary::Div,
            ast::Binary::Mod => Binary::Mod,
            ast::Binary::Add => Binary::Add,
            ast::Binary::Sub => Binary::Sub,
            ast::Binary::And => Binary::And,
            ast::Binary::Or => Binary::Or,
            binary => panic!(
                "[INTERNAL ERROR]: converting {:?} to IR binary operator",
                binary
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    Lt,
    Le,
    Ge,
    Gt,
    Ne,
    Eq,

    /// Unsigned above or equal (http://www.unixwiz.net/techtips/x86-jumps.html)
    ///
    /// Used for optimizing (0 <= signed < max) into (unsigned < max),
    /// relying on integer underflow.
    Ae,
}

impl From<ast::Binary> for Condition {
    fn from(binary: ast::Binary) -> Self {
        match binary {
            ast::Binary::Lt => Condition::Lt,
            ast::Binary::Le => Condition::Le,
            ast::Binary::Ge => Condition::Ge,
            ast::Binary::Gt => Condition::Gt,
            ast::Binary::Ne => Condition::Ne,
            ast::Binary::Eq => Condition::Eq,
            binary => panic!("[INTERNAL ERROR]: converting {:?} to IR condition", binary),
        }
    }
}

#[doc(hidden)]
#[rustfmt::skip]
#[macro_export]
macro_rules! ir {
    (ADD) => { $crate::data::ir::Binary::Add };
    (SUB) => { $crate::data::ir::Binary::Sub };
    (MUL) => { $crate::data::ir::Binary::Mul };
    (HUL) => { $crate::data::ir::Binary::Hul };
    (DIV) => { $crate::data::ir::Binary::Div };
    (MOD) => { $crate::data::ir::Binary::Mod };
    (XOR) => { $crate::data::ir::Binary::Xor };
    (AND) => { $crate::data::ir::Binary::And };
    (OR) => { $crate::data::ir::Binary::Or };

    (LT) => { $crate::data::ir::Condition::Lt };
    (LE) => { $crate::data::ir::Condition::Le };
    (GE) => { $crate::data::ir::Condition::Ge };
    (GT) => { $crate::data::ir::Condition::Gt };
    (NE) => { $crate::data::ir::Condition::Ne };
    (EQ) => { $crate::data::ir::Condition::Eq };
    (AE) => { $crate::data::ir::Condition::Ae };

    ($ident:ident) => { $ident };
}

// https://github.com/rust-lang/rust/pull/52234#issuecomment-976702997
#[doc(hidden)]
pub use ir;
