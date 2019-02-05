use crate::symbol;

/// Represents a Xi source file.
pub struct Program {
    uses: Vec<symbol::Symbol>,
    funs: Vec<Fun>,
}

/// Represents a Xi interface file.
pub struct Interface {
    sigs: Vec<Sig>,
}

/// Represents a function signature (i.e. without implementation).
pub struct Sig {
    name: symbol::Symbol,
    args: Vec<Dec>,
    rets: Vec<Typ>,
}

/// Represents a function definition (i.e. with implementation).
pub struct Fun {
    name: symbol::Symbol,
    args: Vec<Dec>,
    rets: Vec<Typ>,
    body: Stm,
}

/// Represents a primitive type.
pub enum Typ {
    Bit,
    Int,
    Arr(Box<Typ>),
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
    Bit(bool),

    /// Char literal
    Chr(char),

    /// String literal
    Str(String),

    /// Integer literal
    Int(i64),

    /// Variable
    Var(symbol::Symbol),

    /// Array literal
    Arr(Vec<Exp>),

    /// Binary operation
    Bin(Bin, Box<Exp>, Box<Exp>),

    /// Unary operation
    Uno(Uno, Box<Exp>),

    /// Function call
    Fun(symbol::Symbol, Vec<Exp>),

    /// Array index
    Idx(Box<Exp>, Box<Exp>),
}

/// Represents a variable declaration.
pub struct Dec {
    var: symbol::Symbol,
    typ: Typ,
}

/// Represents an imperative statement.
pub enum Stm {
    /// Assignment
    Ass {
        lhs: Vec<Option<Dec>>,
        rhs: Exp,
    },

    /// Procedure call
    Call(symbol::Symbol, Vec<Exp>),

    /// Variable declaration
    Dec(Dec),
    
    /// Return statement
    Ret(Option<Exp>),

    /// Statement block
    Seq(Vec<Stm>),

    /// If-else block
    If {
        cond: Exp,
        pass: Box<Stm>,
        fail: Option<Box<Stm>>,
    },

    /// While block
    While {
        cond: Exp,
        body: Box<Stm>,
    },
}
