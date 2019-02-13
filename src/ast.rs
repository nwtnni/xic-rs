use crate::span::Span;
use crate::symbol;

/// Represents a Xi interface file.
#[derive(Clone, Debug)]
pub struct Interface {
    sigs: Vec<Sig>,
}

/// Represents a Xi source file.
#[derive(Clone, Debug)]
pub struct Program {
    uses: Vec<Use>,
    funs: Vec<Fun>,
}

/// Represents a use statement for importing interfaces.
#[derive(Clone, Debug)]
pub struct Use {
    name: symbol::Symbol,
    span: Span,
}

/// Represents a function signature (i.e. without implementation).
#[derive(Clone, Debug)]
pub struct Sig {
    name: symbol::Symbol,
    args: Vec<Dec>,
    rets: Vec<Typ>,
    span: Span,
}

/// Represents a function definition (i.e. with implementation).
#[derive(Clone, Debug)]
pub struct Fun {
    name: symbol::Symbol,
    args: Vec<Dec>,
    rets: Vec<Typ>,
    body: Stm,
    span: Span,
}

/// Represents a primitive type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Typ {
    Bool(Span),
    Int(Span),
    Arr(Box<Typ>, Span),
}

/// Represents a binary operator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bin {
    Mul,
    Hul,
    Div,
    Mod,
    Add,
    Sub,
    Lt,
    Le,
    Ge,
    Gt,
    Eq,
    Ne,
    And,
    Or,
}

/// Represents a unary operator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Uno {
    Neg,
    Not,
}

/// Represents an expression (i.e. a term that can be evaluated).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Exp {
    /// Boolean literal
    Bool(bool, Span),

    /// Char literal
    Chr(char, Span),

    /// String literal
    Str(String, Span),

    /// Integer literal
    Int(i64, Span),

    /// Variable
    Var(symbol::Symbol, Span),

    /// Array literal
    Arr(Vec<Exp>, Span),

    /// Binary operation
    Bin(Bin, Box<Exp>, Box<Exp>, Span),

    /// Unary operation
    Uno(Uno, Box<Exp>, Span),

    /// Array index
    Idx(Box<Exp>, Box<Exp>, Span),

    /// Function call
    Call(symbol::Symbol, Vec<Exp>, Span),
}

/// Represents a variable declaration.
#[derive(Clone, Debug)]
pub struct Dec {
    name: symbol::Symbol,
    typ: Typ,
    span: Span,
}

/// Represents an imperative statement.
#[derive(Clone, Debug)]
pub enum Stm {
    /// Assignment
    Ass(Vec<Option<Dec>>, Exp, Span),

    /// Procedure call
    Call(symbol::Symbol, Vec<Exp>, Span),

    /// Variable declaration
    Dec(Dec, Span),
    
    /// Return statement
    Ret(Option<Exp>, Span),

    /// Statement block
    Seq(Vec<Stm>, Span),

    /// If-else block
    If(Exp, Box<Stm>, Option<Box<Stm>>, Span),

    /// While block
    While(Exp, Box<Stm>, Span),
}
