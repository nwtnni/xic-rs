mod visit;

pub use visit::Recurse;
pub use visit::VisitorMut;

use std::cell::Cell;
use std::fmt;
use std::iter;

use crate::data::sexp::Serialize as _;
use crate::data::span::Span;
use crate::data::symbol::Symbol;
use crate::util::Or;

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
    pub name: Identifier,
    pub span: Span,
}

impl fmt::Display for Use {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 184] = [0; std::mem::size_of::<ItemSignature>()];

#[derive(Clone, Debug)]
pub enum ItemSignature {
    Class(ClassSignature),
    ClassTemplate(ClassTemplate),
    Function(FunctionSignature),
    FunctionTemplate(FunctionTemplate),
}

impl fmt::Display for ItemSignature {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 184] = [0; std::mem::size_of::<Item>()];

#[derive(Clone, Debug)]
pub enum Item {
    Global(Global),
    Class(Class),
    ClassTemplate(ClassTemplate),
    Function(Function),
    FunctionTemplate(FunctionTemplate),
}

impl fmt::Display for Item {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 72] = [0; std::mem::size_of::<Global>()];

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

const _: [u8; 56] = [0; std::mem::size_of::<Initialization>()];

#[derive(Clone, Debug)]
pub struct Initialization {
    pub declarations: Vec<Option<SingleDeclaration>>,
    pub expression: Box<Expression>,
    pub span: Span,
}

impl fmt::Display for Initialization {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 88] = [0; std::mem::size_of::<ClassTemplate>()];

#[derive(Clone, Debug)]
pub struct ClassTemplate {
    pub name: Identifier,
    pub generics: Vec<Identifier>,
    pub items: Vec<ClassItem>,
    pub span: Span,
}

impl fmt::Display for ClassTemplate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct ClassSignature {
    pub name: Identifier,
    pub extends: Option<Identifier>,
    pub methods: Vec<FunctionSignature>,
    pub span: Span,
}

impl fmt::Display for ClassSignature {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 80] = [0; std::mem::size_of::<Class>()];

#[derive(Clone, Debug)]
pub struct Class {
    pub name: Identifier,
    pub extends: Option<Identifier>,
    pub items: Vec<ClassItem>,
    pub span: Span,
}

impl fmt::Display for Class {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 160] = [0; std::mem::size_of::<ClassItem>()];

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
    fn name(&self) -> &Identifier;
    fn parameters(&self) -> &[SingleDeclaration];
    fn returns(&self) -> &[Type];
}

macro_rules! impl_callable {
    ($type:ty) => {
        impl Callable for $type {
            fn name(&self) -> &Identifier {
                &self.name
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

const _: [u8; 176] = [0; std::mem::size_of::<FunctionTemplate>()];

#[derive(Clone, Debug)]
pub struct FunctionTemplate {
    pub name: Identifier,
    pub generics: Vec<Identifier>,
    pub parameters: Vec<SingleDeclaration>,
    pub returns: Vec<Type>,
    pub statements: Statement,
    pub span: Span,
}

impl fmt::Display for FunctionTemplate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 88] = [0; std::mem::size_of::<FunctionSignature>()];

/// Represents a function signature (i.e. without implementation).
#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub name: Identifier,
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

const _: [u8; 152] = [0; std::mem::size_of::<Function>()];

/// Represents a function definition (i.e. with implementation).
#[derive(Clone, Debug)]
pub struct Function {
    pub name: Identifier,
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

const _: [u8; 72] = [0; std::mem::size_of::<Type>()];

/// Represents a primitive type.
#[derive(Clone, Debug)]
pub enum Type {
    Bool(Span),
    Int(Span),
    Class(Variable),
    Array(Box<Type>, Option<Box<Expression>>, Span),
}

impl Type {
    pub fn has_length(&self) -> bool {
        match self {
            Type::Bool(_) | Type::Int(_) => false,
            Type::Class(variable) => variable.has_length(),
            Type::Array(r#type, length, _) => length.is_some() || r#type.has_length(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Type::Bool(span) | Type::Int(span) | Type::Array(_, _, span) => *span,
            Type::Class(variable) => variable.span,
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

const _: [u8; 64] = [0; std::mem::size_of::<Statement>()];

/// Represents an imperative statement.
#[derive(Clone, Debug)]
pub enum Statement {
    /// Assignment
    Assignment(Box<Expression>, Box<Expression>, Span),

    /// Procedure call
    Call(Call),

    /// Initialization
    Initialization(Initialization),

    /// Variable declaration
    Declaration(Box<Declaration>, Span),

    /// Return statement
    Return(Vec<Expression>, Span),

    /// Statement block
    Sequence(Vec<Statement>, Span),

    /// If-else block
    If(
        Box<Expression>,
        Box<Statement>,
        Option<Box<Statement>>,
        Span,
    ),

    /// While block
    While(Do, Box<Expression>, Box<Statement>, Span),

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

const _: [u8; 96] = [0; std::mem::size_of::<Expression>()];

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

    /// Superclass reference
    Super(Span),

    /// Variable
    Variable(Variable),

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
    Dot(Cell<Option<Symbol>>, Box<Expression>, Identifier, Span),

    /// Class constructor
    New(Variable, Span),
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
            | Expression::Super(span)
            | Expression::Array(_, span)
            | Expression::Binary(_, _, _, span)
            | Expression::Unary(_, _, span)
            | Expression::Index(_, _, span)
            | Expression::Length(_, span)
            | Expression::Dot(_, _, _, span)
            | Expression::New(_, span) => *span,
            Expression::Variable(variable) => variable.span,
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
            | Expression::Super(span)
            | Expression::Array(_, span)
            | Expression::Binary(_, _, _, span)
            | Expression::Unary(_, _, span)
            | Expression::Index(_, _, span)
            | Expression::Length(_, span)
            | Expression::Dot(_, _, _, span)
            | Expression::New(_, span) => span,
            Expression::Variable(variable) => &mut variable.span,
            Expression::Call(call) => &mut call.span,
        }
    }

    /// Assumes this expression represents a Boolean, and negates it. In particular,
    /// we assume that variables are of type Boolean--otherwise, non-boolean expressions
    /// are left unchanged.
    ///
    /// Should only be called after type-checking confirms that `self` is indeed Boolean.
    pub(crate) fn negate_logical(&self) -> Self {
        match self {
            Expression::Variable(variable) => {
                Expression::Unary(Unary::Not, Box::new(self.clone()), variable.span)
            }
            Expression::Boolean(bool, span) => Expression::Boolean(!bool, *span),
            Expression::Binary(binary, left, right, span) => {
                let binary = match binary.get() {
                    Binary::Mul
                    | Binary::Hul
                    | Binary::Div
                    | Binary::Mod
                    | Binary::Add
                    | Binary::Cat
                    | Binary::Sub => return self.clone(),
                    Binary::Lt => Binary::Ge,
                    Binary::Le => Binary::Gt,
                    Binary::Ge => Binary::Lt,
                    Binary::Gt => Binary::Le,
                    Binary::Eq => Binary::Ne,
                    Binary::Ne => Binary::Eq,
                    Binary::And | Binary::Or => {
                        // Alternatively, could use De Morgan's laws, but that would require
                        // recursing on `left` and `right`.
                        return Expression::Unary(Unary::Not, Box::new(self.clone()), *span);
                    }
                };

                Expression::Binary(Cell::new(binary), left.clone(), right.clone(), *span)
            }
            Expression::Unary(Unary::Not, expression, _) => (**expression).clone(),
            Expression::Unary(Unary::Neg, _, _)
            | Expression::Character(_, _)
            | Expression::String(_, _)
            | Expression::Integer(_, _)
            | Expression::Null(_)
            | Expression::This(_)
            | Expression::Super(_)
            | Expression::Array(_, _)
            | Expression::Index(_, _, _)
            | Expression::Length(_, _)
            | Expression::Call(_)
            | Expression::Dot(_, _, _, _)
            | Expression::New(_, _) => self.clone(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [u8; 56] = [0; std::mem::size_of::<Call>()];

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

const _: [u8; 64] = [0; std::mem::size_of::<Declaration>()];

#[derive(Clone, Debug)]
pub enum Declaration {
    Multiple(MultipleDeclaration),
    Single(SingleDeclaration),
}

impl Declaration {
    pub fn iter(&self) -> impl Iterator<Item = (&'_ Identifier, &'_ Type)> + '_ {
        match self {
            Declaration::Single(declaration) => {
                Or::L(iter::once((&declaration.name, &*declaration.r#type)))
            }
            Declaration::Multiple(declaration) => Or::R(
                declaration
                    .names
                    .iter()
                    .map(|name| (name, &*declaration.r#type)),
            ),
        }
    }

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

const _: [u8; 56] = [0; std::mem::size_of::<MultipleDeclaration>()];

#[derive(Clone, Debug)]
pub struct MultipleDeclaration {
    pub names: Vec<Identifier>,
    pub r#type: Box<Type>,
    pub span: Span,
}

impl MultipleDeclaration {
    pub fn new(names: Vec<Identifier>, r#type: Type, span: Span) -> Self {
        Self {
            names,
            r#type: Box::new(r#type),
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

const _: [u8; 48] = [0; std::mem::size_of::<SingleDeclaration>()];

#[derive(Clone, Debug)]
pub struct SingleDeclaration {
    pub name: Identifier,
    pub r#type: Box<Type>,
    pub span: Span,
}

impl SingleDeclaration {
    pub fn new(name: Identifier, r#type: Type, span: Span) -> Self {
        Self {
            name,
            r#type: Box::new(r#type),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    pub name: Identifier,
    pub generics: Option<Vec<Type>>,
    pub span: Span,
}

impl Variable {
    pub fn has_length(&self) -> bool {
        self.generics
            .as_ref()
            .map_or(false, |r#type| r#type.iter().any(Type::has_length))
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Identifier {
    pub symbol: Symbol,
    pub span: Box<Span>,
}

impl fmt::Display for Identifier {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.symbol.cmp(&other.symbol)
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.symbol.cmp(&other.symbol))
    }
}

impl std::hash::Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
    }
}
