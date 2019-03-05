use crate::util::span;
use crate::util::symbol;

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
    pub span: span::Span,
}

/// Represents a function signature (i.e. without implementation).
#[derive(Clone, Debug)]
pub struct Sig {
    pub name: symbol::Symbol,
    pub args: Vec<Dec>,
    pub rets: Vec<Typ>,
    pub span: span::Span,
}

/// Represents a function definition (i.e. with implementation).
#[derive(Clone, Debug)]
pub struct Fun {
    pub name: symbol::Symbol,
    pub args: Vec<Dec>,
    pub rets: Vec<Typ>,
    pub body: Stm,
    pub span: span::Span,
}

/// Represents a primitive type.
#[derive(Clone, Debug)]
pub enum Typ {
    Bool(span::Span),
    Int(span::Span),
    Arr(Box<Typ>, Option<Exp>, span::Span),
}

impl Typ {
    pub fn has_len(&self) -> bool {
        match self {
        | Typ::Bool(_)
        | Typ::Int(_) => false,
        | Typ::Arr(_, Some(_), _) => true,
        | Typ::Arr(typ, _, _) => typ.has_len(),
        }
    }
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

impl Bin {
    pub fn is_numeric(&self) -> bool {
        match self {
        | Bin::Mul | Bin::Hul
        | Bin::Div | Bin::Div
        | Bin::Add | Bin::Sub => true,
        | _ => false,
        }
    }

    pub fn is_compare(&self) -> bool {
        match self {
        | Bin::Lt | Bin::Le
        | Bin::Ge | Bin::Gt
        | Bin::Ne | Bin::Eq => true,
        | _ => false,
        }
    }

    pub fn is_logical(&self) -> bool {
        match self {
        | Bin::And | Bin::Or => true,
        | _ => false,
        }
    }
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
    Bool(bool, span::Span),

    /// Char literal
    Chr(char, span::Span),

    /// String literal
    Str(String, span::Span),

    /// Integer literal
    Int(i64, span::Span),

    /// Variable
    Var(symbol::Symbol, span::Span),

    /// Array literal
    Arr(Vec<Exp>, span::Span),

    /// Binary operation
    Bin(Bin, Box<Exp>, Box<Exp>, span::Span),

    /// Unary operation
    Uno(Uno, Box<Exp>, span::Span),

    /// Array index
    Idx(Box<Exp>, Box<Exp>, span::Span),

    /// Function call
    Call(Call),
}

impl Exp {
    pub fn span(&self) -> span::Span {
        match self {
        | Exp::Bool(_, span)
        | Exp::Chr(_, span)
        | Exp::Str(_, span)
        | Exp::Int(_, span)
        | Exp::Var(_, span)
        | Exp::Arr(_, span)
        | Exp::Bin(_, _, _, span)
        | Exp::Uno(_, _, span)
        | Exp::Idx(_, _, span) => *span,
        | Exp::Call(call) => call.span,
        }
    }
}

/// Represents a variable declaration.
#[derive(Clone, Debug)]
pub struct Dec {
    pub name: symbol::Symbol,
    pub typ: Typ,
    pub span: span::Span,
}

/// Represents a function call.
#[derive(Clone, Debug)]
pub struct Call {
    pub name: symbol::Symbol,
    pub args: Vec<Exp>,
    pub span: span::Span,
}

/// Represents an imperative statement.
#[derive(Clone, Debug)]
pub enum Stm {
    /// Assignment
    Ass(Exp, Exp, span::Span),

    /// Procedure call
    Call(Call),

    /// Initialization
    Init(Vec<Option<Dec>>, Exp, span::Span),

    /// Variable declaration
    Dec(Dec, span::Span),
    
    /// Return statement
    Ret(Vec<Exp>, span::Span),

    /// Statement block
    Seq(Vec<Stm>, span::Span),

    /// If-else block
    If(Exp, Box<Stm>, Option<Box<Stm>>, span::Span),

    /// While block
    While(Exp, Box<Stm>, span::Span),
}

impl Stm {
    pub fn span(&self) -> span::Span {
        match self {
        | Stm::Call(call) => call.span,
        | Stm::Ass(_, _, span)
        | Stm::Init(_, _, span)
        | Stm::Dec(_, span)
        | Stm::Ret(_, span)
        | Stm::Seq(_, span)
        | Stm::If(_, _, _, span)
        | Stm::While(_, _, span) => *span,
        }
    }
}
