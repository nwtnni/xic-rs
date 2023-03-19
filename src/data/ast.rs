mod visit;

pub use visit::VisitorMut;

use std::cell::Cell;
use std::fmt;
use std::hash;
use std::iter;

use crate::data::r#type;
use crate::data::sexp::Serialize as _;
use crate::data::span::Span;
use crate::data::symbol::Symbol;
use crate::util::Or;

/// Represents a Xi interface file.
#[derive(Clone, Debug)]
pub struct Interface<T> {
    pub uses: Vec<Use>,
    pub items: Vec<ItemSignature<T>>,
}

impl<T> fmt::Display for Interface<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

/// Represents a Xi source file.
#[derive(Clone, Debug)]
pub struct Program<T> {
    pub uses: Vec<Use>,
    pub items: Vec<Item<T>>,
}

impl<T> fmt::Display for Program<T> {
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

const _: [(); 176] = [(); std::mem::size_of::<ItemSignature<()>>()];

#[derive(Clone, Debug)]
pub enum ItemSignature<T> {
    Class(ClassSignature<T>),
    ClassTemplate(ClassTemplate),
    Function(FunctionSignature<T>),
    FunctionTemplate(FunctionTemplate),
}

impl<T> fmt::Display for ItemSignature<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 184] = [(); std::mem::size_of::<Item<()>>()];

#[derive(Clone, Debug)]
pub enum Item<T> {
    Global(Global<T>),
    Class(Class<T>),
    ClassTemplate(ClassTemplate),
    Function(Function<T>),
    FunctionTemplate(FunctionTemplate),
}

impl<T> fmt::Display for Item<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 64] = [(); std::mem::size_of::<Global<()>>()];

#[derive(Clone, Debug)]
pub enum Global<T> {
    Declaration(Declaration<T>),
    Initialization(Initialization<T>),
}

impl<T> fmt::Display for Global<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 56] = [(); std::mem::size_of::<Initialization<()>>()];

#[derive(Clone, Debug)]
pub struct Initialization<T> {
    pub declarations: Vec<Option<SingleDeclaration<T>>>,
    pub expression: Box<Expression<T>>,
    pub span: Span,
}

impl<T> fmt::Display for Initialization<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 160] = [(); std::mem::size_of::<ClassTemplate>()];

pub trait ClassLike<T> {
    fn r#final(&self) -> bool;
    fn name(&self) -> &Identifier;
    fn extends(&self) -> Option<&Variable<T>>;
}

macro_rules! impl_class_like {
    ($type:ident) => {
        impl<T> ClassLike<T> for $type<T> {
            fn r#final(&self) -> bool {
                self.r#final
            }

            fn name(&self) -> &Identifier {
                &self.name
            }

            fn extends(&self) -> Option<&Variable<T>> {
                self.extends.as_ref()
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct ClassTemplate {
    pub r#final: bool,
    pub name: Identifier,
    pub generics: Vec<Identifier>,
    pub extends: Option<Variable<()>>,
    pub items: Vec<ClassItem<()>>,
    pub span: Span,
}

impl ClassTemplate {
    #[must_use]
    pub fn new(
        r#final: bool,
        name: Identifier,
        generics: Vec<Identifier>,
        extends: Option<Variable<()>>,
        items: Vec<ClassItem<()>>,
        span: Span,
    ) -> Self {
        Self {
            r#final,
            name,
            generics,
            extends,
            items,
            span,
        }
    }
}

impl fmt::Display for ClassTemplate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub struct ClassSignature<T> {
    pub r#final: bool,
    pub name: Identifier,
    pub extends: Option<Variable<T>>,
    pub methods: Vec<FunctionSignature<T>>,
    pub span: Span,
}

impl_class_like!(ClassSignature);

impl ClassSignature<()> {
    #[must_use]
    pub fn new(
        r#final: bool,
        name: Identifier,
        extends: Option<Variable<()>>,
        methods: Vec<FunctionSignature<()>>,
        span: Span,
    ) -> Self {
        Self {
            r#final,
            name,
            extends,
            methods,
            span,
        }
    }
}

impl<T> fmt::Display for ClassSignature<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 160] = [(); std::mem::size_of::<Class<()>>()];

#[derive(Clone, Debug)]
pub struct Class<T> {
    pub r#final: bool,
    pub name: Identifier,
    pub extends: Option<Variable<T>>,
    pub items: Vec<ClassItem<T>>,
    // Used for tracking class template instantiation chains in diagnostics
    pub(crate) provenance: Vec<Span>,
    /// Whether this class is declared in an interface
    pub(crate) declared: Cell<bool>,
    pub span: Span,
}

impl_class_like!(Class);

impl Class<()> {
    #[must_use]
    pub fn new(
        r#final: bool,
        name: Identifier,
        extends: Option<Variable<()>>,
        items: Vec<ClassItem<()>>,
        provenance: Vec<Span>,
        span: Span,
    ) -> Self {
        Self {
            r#final,
            name,
            extends,
            items,
            provenance,
            declared: Cell::new(false),
            span,
        }
    }
}

impl<T> fmt::Display for Class<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 184] = [(); std::mem::size_of::<ClassItem<()>>()];

#[derive(Clone, Debug)]
pub enum ClassItem<T> {
    Field(Declaration<T>),
    Method(Function<T>),
}

impl<T> fmt::Display for ClassItem<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

pub trait FunctionLike<T> {
    fn name(&self) -> &Identifier;
    fn parameters(&self) -> &[SingleDeclaration<T>];
    fn returns(&self) -> &[Type<T>];
}

macro_rules! impl_function_like {
    ($type:ident) => {
        impl<T> FunctionLike<T> for $type<T> {
            fn name(&self) -> &Identifier {
                &self.name
            }

            fn parameters(&self) -> &[SingleDeclaration<T>] {
                &self.parameters
            }

            fn returns(&self) -> &[Type<T>] {
                &self.returns
            }
        }
    };
}

const _: [(); 176] = [(); std::mem::size_of::<FunctionTemplate>()];

#[derive(Clone, Debug)]
pub struct FunctionTemplate {
    pub name: Identifier,
    pub generics: Vec<Identifier>,
    pub parameters: Vec<SingleDeclaration<()>>,
    pub returns: Vec<Type<()>>,
    pub statements: Statement<()>,
    pub span: Span,
}

impl fmt::Display for FunctionTemplate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 88] = [(); std::mem::size_of::<FunctionSignature<()>>()];

/// Represents a function signature (i.e. without implementation).
#[derive(Clone, Debug)]
pub struct FunctionSignature<T> {
    pub name: Identifier,
    pub parameters: Vec<SingleDeclaration<T>>,
    pub returns: Vec<Type<T>>,
    pub span: Span,
}

impl_function_like!(FunctionSignature);

impl<T> fmt::Display for FunctionSignature<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 184] = [(); std::mem::size_of::<Function<()>>()];

/// Represents a function definition (i.e. with implementation).
#[derive(Clone, Debug)]
pub struct Function<T> {
    pub name: Identifier,
    pub parameters: Vec<SingleDeclaration<T>>,
    pub returns: Vec<Type<T>>,
    pub statements: Statement<T>,
    // Used for tracking function template instantiation chains in diagnostics
    pub(crate) provenance: Vec<Span>,
    /// Whether this function is declared in an interface
    pub(crate) declared: Cell<bool>,
    pub span: Span,
}

impl_function_like!(Function);

impl<T> fmt::Display for Function<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 72] = [(); std::mem::size_of::<Type<()>>()];

/// Represents a primitive type.
#[derive(Clone, Debug)]
pub enum Type<T> {
    Bool(Span),
    Int(Span),
    Class(Variable<T>),
    Array(Box<Type<T>>, Option<Box<Expression<T>>>, Span),
}

impl<T> Type<T> {
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

impl<T> PartialEq for Type<T> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Type::Bool(_) => matches!(other, Type::Bool(_)),
            Type::Int(_) => matches!(other, Type::Int(_)),
            Type::Class(lhs) => matches!(other, Type::Class(rhs) if lhs == rhs),
            // Note: ignores array length expression
            Type::Array(lhs, _, _) => matches!(other, Type::Array(rhs, _, _) if lhs == rhs),
        }
    }
}

impl<T> Eq for Type<T> {}

impl<T> hash::Hash for Type<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);

        match self {
            Type::Bool(_) | Type::Int(_) => (),
            Type::Class(variable) => variable.hash(state),
            // Note: ignores array length expression
            Type::Array(r#type, _, _) => r#type.hash(state),
        }
    }
}

impl<T> fmt::Display for Type<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 64] = [(); std::mem::size_of::<Statement<()>>()];

/// Represents an imperative statement.
#[derive(Clone, Debug)]
pub enum Statement<T> {
    /// Assignment
    Assignment(Box<Expression<T>>, Box<Expression<T>>, Span),

    /// Procedure call
    Call(Call<T>),

    /// Initialization
    Initialization(Initialization<T>),

    /// Variable declaration
    Declaration(Box<Declaration<T>>, Span),

    /// Return statement
    Return(Vec<Expression<T>>, Span),

    /// Statement block
    Sequence(Vec<Statement<T>>, Span),

    /// If-else block
    If(
        Box<Expression<T>>,
        Box<Statement<T>>,
        Option<Box<Statement<T>>>,
        Span,
    ),

    /// While block
    While(Do, Box<Expression<T>>, Box<Statement<T>>, Span),

    /// Break statement
    Break(Span),
}

impl<T> Statement<T> {
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

impl<T> fmt::Display for Statement<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

#[derive(Clone, Debug)]
pub enum Do {
    Yes,
    No,
}

const _: [(); 96] = [(); std::mem::size_of::<Expression<()>>()];

/// Represents an expression (i.e. a term that can be evaluated).
#[derive(Clone, Debug)]
pub enum Expression<T> {
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
    This(T, Span),

    /// Superclass reference
    Super(T, Span),

    /// Variable
    Variable(Variable<T>, T),

    /// Array literal
    Array(Vec<Expression<T>>, T, Span),

    /// Binary operation
    Binary(Binary, Box<Expression<T>>, Box<Expression<T>>, T, Span),

    /// Unary operation
    Unary(Unary, Box<Expression<T>>, T, Span),

    /// Array index
    Index(Box<Expression<T>>, Box<Expression<T>>, T, Span),

    /// Array length
    Length(Box<Expression<T>>, Span),

    /// Function call
    Call(Call<T>),

    /// Dot operator
    ///
    /// FIXME: remove `Cell<Option<Symbol>>`
    Dot(Box<Expression<T>>, Identifier, T, Span),

    /// Class constructor
    New(Variable<T>, Span),
}

impl<T> Expression<T> {
    pub fn span(&self) -> Span {
        match self {
            Expression::Boolean(_, span)
            | Expression::Character(_, span)
            | Expression::String(_, span)
            | Expression::Integer(_, span)
            | Expression::Null(span)
            | Expression::This(_, span)
            | Expression::Super(_, span)
            | Expression::Array(_, _, span)
            | Expression::Binary(_, _, _, _, span)
            | Expression::Unary(_, _, _, span)
            | Expression::Index(_, _, _, span)
            | Expression::Length(_, span)
            | Expression::Dot(_, _, _, span)
            | Expression::New(_, span) => *span,
            Expression::Variable(variable, _) => variable.span,
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
            | Expression::This(_, span)
            | Expression::Super(_, span)
            | Expression::Array(_, _, span)
            | Expression::Binary(_, _, _, _, span)
            | Expression::Unary(_, _, _, span)
            | Expression::Index(_, _, _, span)
            | Expression::Length(_, span)
            | Expression::Dot(_, _, _, span)
            | Expression::New(_, span) => span,
            Expression::Variable(variable, _) => &mut variable.span,
            Expression::Call(call) => &mut call.span,
        }
    }
}

impl Expression<r#type::Expression> {
    /// Assumes this expression represents a Boolean, and negates it. In particular,
    /// we assume that variables are of type Boolean--otherwise, non-boolean expressions
    /// are left unchanged.
    ///
    /// Should only be called after type-checking confirms that `self` is indeed Boolean.
    pub(crate) fn negate_logical(&self) -> Self {
        match self {
            Expression::Variable(variable, r#type) => Expression::Unary(
                Unary::Not,
                Box::new(self.clone()),
                r#type.clone(),
                variable.span,
            ),
            Expression::Boolean(bool, span) => Expression::Boolean(!bool, *span),
            Expression::Binary(binary, left, right, r#type, span) => {
                let binary = match binary {
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
                        return Expression::Unary(
                            Unary::Not,
                            Box::new(self.clone()),
                            r#type.clone(),
                            *span,
                        );
                    }
                };

                Expression::Binary(binary, left.clone(), right.clone(), r#type.clone(), *span)
            }
            Expression::Unary(Unary::Not, expression, _, _) => (**expression).clone(),
            Expression::Index(_, _, _, _) | Expression::Call(_) | Expression::Dot(_, _, _, _) => {
                Expression::Unary(
                    Unary::Not,
                    Box::new(self.clone()),
                    r#type::Expression::Boolean,
                    self.span(),
                )
            }
            Expression::Unary(Unary::Neg, _, _, _)
            | Expression::Character(_, _)
            | Expression::String(_, _)
            | Expression::Integer(_, _)
            | Expression::Null(_)
            | Expression::This(_, _)
            | Expression::Super(_, _)
            | Expression::Array(_, _, _)
            | Expression::Length(_, _)
            | Expression::New(_, _) => self.clone(),
        }
    }
}

impl<T> fmt::Display for Expression<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 56] = [(); std::mem::size_of::<Call<()>>()];

/// Represents a function call.
#[derive(Clone, Debug)]
pub struct Call<T> {
    pub function: Box<Expression<T>>,
    pub arguments: Vec<Expression<T>>,
    pub span: Span,
}

impl<T> fmt::Display for Call<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 64] = [(); std::mem::size_of::<Declaration<()>>()];

#[derive(Clone, Debug)]
pub enum Declaration<T> {
    Multiple(MultipleDeclaration<T>),
    Single(SingleDeclaration<T>),
}

impl<T> Declaration<T> {
    pub fn iter(&self) -> impl Iterator<Item = (&'_ Identifier, &'_ Type<T>)> + '_ {
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

impl<T> fmt::Display for Declaration<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 56] = [(); std::mem::size_of::<MultipleDeclaration<()>>()];

#[derive(Clone, Debug)]
pub struct MultipleDeclaration<T> {
    pub names: Vec<Identifier>,
    pub r#type: Box<Type<T>>,
    pub span: Span,
}

impl<T> MultipleDeclaration<T> {
    pub fn new(names: Vec<Identifier>, r#type: Type<T>, span: Span) -> Self {
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

impl<T> fmt::Display for MultipleDeclaration<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sexp())
    }
}

const _: [(); 48] = [(); std::mem::size_of::<SingleDeclaration<()>>()];

#[derive(Clone, Debug)]
pub struct SingleDeclaration<T> {
    pub name: Identifier,
    pub r#type: Box<Type<T>>,
    pub span: Span,
}

impl<T> SingleDeclaration<T> {
    pub fn new(name: Identifier, r#type: Type<T>, span: Span) -> Self {
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

impl<T> fmt::Display for SingleDeclaration<T> {
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

#[derive(Clone, Debug)]
pub struct Variable<T> {
    pub name: Identifier,
    pub generics: Option<Vec<Type<T>>>,
    pub span: Span,
}

impl<T> Variable<T> {
    pub fn has_length(&self) -> bool {
        self.generics
            .as_ref()
            .map_or(false, |r#type| r#type.iter().any(Type::has_length))
    }
}

impl<T> PartialEq for Variable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.generics == other.generics
    }
}

impl<T> Eq for Variable<T> {}

impl<T> hash::Hash for Variable<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.generics.hash(state);
    }
}

impl<T> fmt::Display for Variable<T> {
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
