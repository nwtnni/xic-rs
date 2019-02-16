use crate::span::Span;
use crate::symbol;

/// Represents a Xi interface file.
#[derive(Clone, Debug)]
pub struct Interface {
    pub sigs: Vec<Sig>,
}

/// Represents a Xi source file.
#[derive(Clone, Debug)]
pub struct Program {
    pub uses: Vec<Use>,
    pub funs: Vec<Fun>,
}

/// Represents a use statement for importing interfaces.
#[derive(Clone, Debug)]
pub struct Use {
    pub name: symbol::Symbol,
    pub span: Span,
}

/// Represents a function signature (i.e. without implementation).
#[derive(Clone, Debug)]
pub struct Sig {
    pub name: symbol::Symbol,
    pub args: Vec<Dec>,
    pub rets: Vec<Typ>,
    pub span: Span,
}

/// Represents a function definition (i.e. with implementation).
#[derive(Clone, Debug)]
pub struct Fun {
    pub name: symbol::Symbol,
    pub args: Vec<Dec>,
    pub rets: Vec<Typ>,
    pub body: Stm,
    pub span: Span,
}

/// Represents a primitive type.
#[derive(Clone, Debug)]
pub enum Typ {
    Bool(Span),
    Int(Span),
    Arr(Box<Typ>, Option<Exp>, Span),
}

impl PartialEq for Typ {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
        | (Typ::Bool(_), Typ::Bool(_))
        | (Typ::Int(_), Typ::Int(_)) => true,
        | (Typ::Arr(lhs, _, _), Typ::Arr(rhs, _, _)) => lhs == rhs,
        | _ => false,
        }
    }
}

impl Eq for Typ {}

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
#[derive(Clone, Debug)]
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
    Call(Call),
}

/// Represents a variable declaration.
#[derive(Clone, Debug)]
pub struct Dec {
    pub name: symbol::Symbol,
    pub typ: Typ,
    pub span: Span,
}

/// Represents a function call.
#[derive(Clone, Debug)]
pub struct Call {
    pub name: symbol::Symbol,
    pub args: Vec<Exp>,
    pub span: Span,
}

/// Represents an imperative statement.
#[derive(Clone, Debug)]
pub enum Stm {
    /// Assignment
    Ass(Exp, Exp, Span),

    /// Procedure call
    Call(Call),

    /// Initialization
    Init(Vec<Option<Dec>>, Exp, Span),

    /// Variable declaration
    Dec(Dec, Span),
    
    /// Return statement
    Ret(Vec<Exp>, Span),

    /// Statement block
    Seq(Vec<Stm>, Span),

    /// If-else block
    If(Exp, Box<Stm>, Option<Box<Stm>>, Span),

    /// While block
    While(Exp, Box<Stm>, Span),
}
