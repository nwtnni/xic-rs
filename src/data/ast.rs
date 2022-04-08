use crate::util::span::Span;
use crate::util::symbol::Symbol;

/// Represents a Xi interface file.
#[derive(Clone, Debug)]
pub struct Interface {
    pub signatures: Vec<Signature>,
}

/// Represents a Xi source file.
#[derive(Clone, Debug)]
pub struct Program {
    pub uses: Vec<Use>,
    pub functions: Vec<Function>,
}

/// Represents a use statement for importing interfaces.
#[derive(Clone, Debug)]
pub struct Use {
    pub name: Symbol,
    pub span: Span,
}

pub trait Callable {
    fn name(&self) -> Symbol;
    fn parameters(&self) -> &[Declaration];
    fn returns(&self) -> &[Type];
}

macro_rules! impl_callable {
    ($type:ty) => {
        impl Callable for $type {
            fn name(&self) -> Symbol {
                self.name
            }

            fn parameters(&self) -> &[Declaration] {
                &self.parameters
            }

            fn returns(&self) -> &[Type] {
                &self.returns
            }
        }
    };
}

/// Represents a function signature (i.e. without implementation).
#[derive(Clone, Debug)]
pub struct Signature {
    pub name: Symbol,
    pub parameters: Vec<Declaration>,
    pub returns: Vec<Type>,
    pub span: Span,
}

impl_callable!(Signature);

/// Represents a function definition (i.e. with implementation).
#[derive(Clone, Debug)]
pub struct Function {
    pub name: Symbol,
    pub parameters: Vec<Declaration>,
    pub returns: Vec<Type>,
    pub statements: Statement,
    pub span: Span,
}

impl_callable!(Function);

/// Represents a primitive type.
#[derive(Clone, Debug)]
pub enum Type {
    Bool(Span),
    Int(Span),
    Array(Box<Type>, Option<Expression>, Span),
}

impl Type {
    pub fn has_len(&self) -> bool {
        match self {
            Type::Bool(_) | Type::Int(_) => false,
            Type::Array(_, Some(_), _) => true,
            Type::Array(typ, _, _) => typ.has_len(),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Bool(_), Type::Bool(_)) | (Type::Int(_), Type::Int(_)) => true,
            (Type::Array(lhs, _, _), Type::Array(rhs, _, _)) => lhs == rhs,
            _ => false,
        }
    }
}

impl Eq for Type {}

/// Represents a binary operator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Binary {
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
pub enum Unary {
    Neg,
    Not,
}

/// Represents an expression (i.e. a term that can be evaluated).
#[derive(Clone, Debug)]
pub enum Expression {
    /// Boolean literal
    Boolean(bool, Span),

    /// Char literal
    Character(char, Span),

    /// String literal
    String(String, Span),

    /// Integer literal
    Integer(i64, Span),

    /// Variable
    Variable(Symbol, Span),

    /// Array literal
    Array(Vec<Expression>, Span),

    /// Binary operation
    Binary(Binary, Box<Expression>, Box<Expression>, Span),

    /// Unary operation
    Unary(Unary, Box<Expression>, Span),

    /// Array index
    Index(Box<Expression>, Box<Expression>, Span),

    /// Function call
    Call(Call),
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Boolean(_, span)
            | Expression::Character(_, span)
            | Expression::String(_, span)
            | Expression::Integer(_, span)
            | Expression::Variable(_, span)
            | Expression::Array(_, span)
            | Expression::Binary(_, _, _, span)
            | Expression::Unary(_, _, span)
            | Expression::Index(_, _, span) => *span,
            Expression::Call(call) => call.span,
        }
    }

    pub fn span_mut(&mut self) -> &mut Span {
        match self {
            Expression::Boolean(_, span)
            | Expression::Character(_, span)
            | Expression::String(_, span)
            | Expression::Integer(_, span)
            | Expression::Variable(_, span)
            | Expression::Array(_, span)
            | Expression::Binary(_, _, _, span)
            | Expression::Unary(_, _, span)
            | Expression::Index(_, _, span) => span,
            Expression::Call(call) => &mut call.span,
        }
    }
}

/// Represents a variable declaration.
#[derive(Clone, Debug)]
pub struct Declaration {
    pub name: Symbol,
    pub r#type: Type,
    pub span: Span,
}

impl Declaration {
    pub fn new(name: Symbol, r#type: Type, span: Span) -> Self {
        Self { name, r#type, span }
    }

    pub fn has_len(&self) -> bool {
        self.r#type.has_len()
    }
}

/// Represents a function call.
#[derive(Clone, Debug)]
pub struct Call {
    pub name: Symbol,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

/// Represents an imperative statement.
#[derive(Clone, Debug)]
pub enum Statement {
    /// Assignment
    Assignment(Expression, Expression, Span),

    /// Procedure call
    Call(Call),

    /// Initialization
    Initialization(Vec<Option<Declaration>>, Expression, Span),

    /// Variable declaration
    Declaration(Declaration, Span),

    /// Return statement
    Return(Vec<Expression>, Span),

    /// Statement block
    Sequence(Vec<Statement>, Span),

    /// If-else block
    If(Expression, Box<Statement>, Option<Box<Statement>>, Span),

    /// While block
    While(Expression, Box<Statement>, Span),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Call(call) => call.span,
            Statement::Assignment(_, _, span)
            | Statement::Initialization(_, _, span)
            | Statement::Declaration(_, span)
            | Statement::Return(_, span)
            | Statement::Sequence(_, span)
            | Statement::If(_, _, _, span)
            | Statement::While(_, _, span) => *span,
        }
    }
}
