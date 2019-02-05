use crate::span::Span;
use crate::symbol;

/// Represents a Xi interface file.
pub struct Interface {
    sigs: Vec<Sig>,
}

/// Represents a Xi source file.
pub struct Program {
    uses: Vec<Use>,
    funs: Vec<Fun>,
}

/// Represents a use statement for importing interfaces.
pub struct Use {
    name: symbol::Symbol,
    span: Span,
}

/// Represents a function signature (i.e. without implementation).
pub struct Sig {
    name: symbol::Symbol,
    args: Vec<Dec>,
    rets: Vec<Typ>,
    span: Span,
}

/// Represents a function definition (i.e. with implementation).
pub struct Fun {
    name: symbol::Symbol,
    args: Vec<Dec>,
    rets: Vec<Typ>,
    body: Stm,
    span: Span,
}

/// Represents a primitive type.
pub enum Typ {
    Bool(Span),
    Int(Span),
    Arr(Box<Typ>, Span),
}

/// Represents a binary operator.
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
pub enum Uno {
    Neg,
    Not,
}

/// Represents an expression (i.e. a term that can be evaluated).
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

    /// Function call
    Fun(symbol::Symbol, Vec<Exp>, Span),

    /// Array index
    Idx(Box<Exp>, Box<Exp>, Span),
}

/// Represents a variable declaration.
pub struct Dec {
    name: symbol::Symbol,
    typ: Typ,
    span: Span,
}

/// Represents an imperative statement.
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
