use std::cell::Cell;
use std::fmt;

use crate::data::sexp::Serialize as _;
use crate::data::span::Span;
use crate::data::symbol::Symbol;

/// Represents a Xi interface file.
#[derive(Clone, Debug)]
pub struct Interface {
    pub uses: Vec<Use>,
    pub items: Vec<ItemSignature>,
}

impl fmt::Display for Interface {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a Xi source file.
#[derive(Clone, Debug)]
pub struct Program {
    pub uses: Vec<Use>,
    pub items: Vec<Item>,
}

impl fmt::Display for Program {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a use statement for importing interfaces.
#[derive(Clone, Debug)]
pub struct Use {
    pub name: Symbol,
    pub span: Span,
}

impl fmt::Display for Use {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub enum ItemSignature {
    Class(ClassSignature),
    Function(FunctionSignature),
}

impl fmt::Display for ItemSignature {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub enum Item {
    Global(Global),
    Class(Class),
    Function(Function),
}

impl fmt::Display for Item {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub enum Global {
    Declaration(Declaration),
    Initialization(Initialization),
}

impl fmt::Display for Global {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct Initialization {
    pub declarations: Vec<Option<SingleDeclaration>>,
    pub expression: Expression,
    pub span: Span,
}

impl fmt::Display for Initialization {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct ClassSignature {
    pub name: Symbol,
    pub extends: Option<Symbol>,
    pub methods: Vec<FunctionSignature>,
    pub span: Span,
}

impl fmt::Display for ClassSignature {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct Class {
    pub name: Symbol,
    pub extends: Option<Symbol>,
    pub items: Vec<ClassItem>,
    pub span: Span,
}

impl fmt::Display for Class {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub enum ClassItem {
    Field(Declaration),
    Method(Function),
}

impl fmt::Display for ClassItem {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

pub trait Callable {
    fn name(&self) -> Symbol;
    fn parameters(&self) -> &[SingleDeclaration];
    fn returns(&self) -> &[Type];
}

macro_rules! impl_callable {
    ($type:ty) => {
        impl Callable for $type {
            fn name(&self) -> Symbol {
                self.name
            }

            fn parameters(&self) -> &[SingleDeclaration] {
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
pub struct FunctionSignature {
    pub name: Symbol,
    pub parameters: Vec<SingleDeclaration>,
    pub returns: Vec<Type>,
    pub span: Span,
}

impl_callable!(FunctionSignature);

impl fmt::Display for FunctionSignature {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a function definition (i.e. with implementation).
#[derive(Clone, Debug)]
pub struct Function {
    pub name: Symbol,
    pub parameters: Vec<SingleDeclaration>,
    pub returns: Vec<Type>,
    pub statements: Statement,
    pub span: Span,
}

impl_callable!(Function);

impl fmt::Display for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a primitive type.
#[derive(Clone, Debug)]
pub enum Type {
    Bool(Span),
    Int(Span),
    Class(Symbol, Span),
    Array(Box<Type>, Option<Expression>, Span),
}

impl Type {
    pub fn has_length(&self) -> bool {
        match self {
            Type::Bool(_) | Type::Int(_) | Type::Class(_, _) => false,
            Type::Array(r#type, length, _) => length.is_some() || r#type.has_length(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Type::Bool(span) | Type::Int(span) | Type::Class(_, span) | Type::Array(_, _, span) => {
                *span
            }
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

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents an imperative statement.
#[derive(Clone, Debug)]
pub enum Statement {
    /// Assignment
    Assignment(Expression, Expression, Span),

    /// Procedure call
    Call(Call),

    /// Initialization
    Initialization(Initialization),

    /// Variable declaration
    Declaration(Declaration, Span),

    /// Return statement
    Return(Vec<Expression>, Span),

    /// Statement block
    Sequence(Vec<Statement>, Span),

    /// If-else block
    If(Expression, Box<Statement>, Option<Box<Statement>>, Span),

    /// While block
    While(Do, Expression, Box<Statement>, Span),

    /// Break statement
    Break(Span),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Call(call) => call.span,
            Statement::Initialization(initialization) => initialization.span,
            Statement::Assignment(_, _, span)
            | Statement::Declaration(_, span)
            | Statement::Return(_, span)
            | Statement::Sequence(_, span)
            | Statement::If(_, _, _, span)
            | Statement::While(_, _, _, span)
            | Statement::Break(span) => *span,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub enum Do {
    Yes,
    No,
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

    /// Null literal
    Null(Span),

    /// Class reference
    This(Span),

    /// Variable
    Variable(Symbol, Span),

    /// Array literal
    Array(Vec<Expression>, Span),

    /// Binary operation
    Binary(Cell<Binary>, Box<Expression>, Box<Expression>, Span),

    /// Unary operation
    Unary(Unary, Box<Expression>, Span),

    /// Array index
    Index(Box<Expression>, Box<Expression>, Span),

    /// Array length
    Length(Box<Expression>, Span),

    /// Function call
    Call(Call),

    /// Dot operator
    Dot(Box<Expression>, Symbol, Span),

    /// Class constructor
    New(Symbol, Span),
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Boolean(_, span)
            | Expression::Character(_, span)
            | Expression::String(_, span)
            | Expression::Integer(_, span)
            | Expression::Null(span)
            | Expression::This(span)
            | Expression::Variable(_, span)
            | Expression::Array(_, span)
            | Expression::Binary(_, _, _, span)
            | Expression::Unary(_, _, span)
            | Expression::Index(_, _, span)
            | Expression::Length(_, span)
            | Expression::Dot(_, _, span)
            | Expression::New(_, span) => *span,
            Expression::Call(call) => call.span,
        }
    }

    pub fn span_mut(&mut self) -> &mut Span {
        match self {
            Expression::Boolean(_, span)
            | Expression::Character(_, span)
            | Expression::String(_, span)
            | Expression::Integer(_, span)
            | Expression::Null(span)
            | Expression::This(span)
            | Expression::Variable(_, span)
            | Expression::Array(_, span)
            | Expression::Binary(_, _, _, span)
            | Expression::Unary(_, _, span)
            | Expression::Index(_, _, span)
            | Expression::Length(_, span)
            | Expression::Dot(_, _, span)
            | Expression::New(_, span) => span,
            Expression::Call(call) => &mut call.span,
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a function call.
#[derive(Clone, Debug)]
pub struct Call {
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

impl fmt::Display for Call {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub enum Declaration {
    Multiple(MultipleDeclaration),
    Single(SingleDeclaration),
}

impl Declaration {
    pub fn has_length(&self) -> bool {
        match self {
            Declaration::Multiple(multiple) => multiple.has_length(),
            Declaration::Single(single) => single.has_length(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Declaration::Multiple(multiple) => multiple.span(),
            Declaration::Single(single) => single.span(),
        }
    }
}

impl fmt::Display for Declaration {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct MultipleDeclaration {
    pub names: Vec<Symbol>,
    pub r#type: Type,
    pub span: Span,
}

impl MultipleDeclaration {
    pub fn new(names: Vec<Symbol>, r#type: Type, span: Span) -> Self {
        Self {
            names,
            r#type,
            span,
        }
    }

    pub fn has_length(&self) -> bool {
        self.r#type.has_length()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for MultipleDeclaration {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct SingleDeclaration {
    pub name: Symbol,
    pub r#type: Type,
    pub span: Span,
}

impl SingleDeclaration {
    pub fn new(name: Symbol, r#type: Type, span: Span) -> Self {
        Self { name, r#type, span }
    }

    pub fn has_length(&self) -> bool {
        self.r#type.has_length()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for SingleDeclaration {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a binary operator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Binary {
    Mul,
    Hul,
    Div,
    Mod,
    Add,
    Cat,
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

impl fmt::Display for Binary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a unary operator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Unary {
    Neg,
    Not,
}

impl fmt::Display for Unary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}
