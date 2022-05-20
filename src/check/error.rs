use std::borrow::Cow;

use crate::data::r#type;
use crate::data::span;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::error;

#[derive(Clone, Debug)]
pub struct Error {
    span: span::Span,
    kind: ErrorKind,
}

impl Error {
    pub fn new(span: span::Span, kind: ErrorKind) -> Self {
        Error { span, kind }
    }

    pub(super) fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound(Symbol),
    UnboundVariable(Symbol),
    UnboundFun(Symbol),
    NotVariable(Symbol),
    NotFun(Option<Symbol>),
    NotExp,
    NotProcedure,
    NotClass,
    NotInClass(Option<Symbol>),
    NotInClassModule(Symbol),
    ClassCycle(Symbol),
    IndexEmpty,
    CallLength,
    InitLength,
    InitProcedure,
    Unreachable,
    MissingReturn,
    ReturnMismatch,
    NameClash,
    SignatureMismatch,
    Mismatch {
        expected: r#type::Expression,
        found: r#type::Expression,
    },
}

impl ErrorKind {
    fn message(&self) -> Cow<'static, str> {
        match self {
            ErrorKind::NotFound(i) => Cow::Owned(format!(
                "Interface file not found in library directory: {}.ixi",
                i
            )),
            ErrorKind::UnboundVariable(v) => {
                Cow::Owned(format!("Unbound variable {}", symbol::resolve(*v)))
            }
            ErrorKind::UnboundFun(f) => {
                Cow::Owned(format!("Unbound function {}", symbol::resolve(*f)))
            }
            ErrorKind::NotVariable(v) => {
                Cow::Owned(format!("{} is not a variable type", symbol::resolve(*v)))
            }
            ErrorKind::NotFun(Some(f)) => {
                Cow::Owned(format!("{} is not a function", symbol::resolve(*f)))
            }
            ErrorKind::NotFun(None) => Cow::Borrowed("Not a function"),
            ErrorKind::NotExp => Cow::Borrowed("Not a single expression type"),
            ErrorKind::NotProcedure => Cow::Borrowed("Not a procedure"),
            ErrorKind::NotClass => Cow::Borrowed("Receiver of dot operator must be a class"),
            ErrorKind::NotInClass(None) => Cow::Borrowed("Not inside a class implementation"),
            ErrorKind::NotInClass(Some(class)) => Cow::Owned(format!("Not inside class {}", class)),
            ErrorKind::NotInClassModule(class) => {
                Cow::Owned(format!("Not inside module that defines class {}", class))
            }
            ErrorKind::ClassCycle(class) => {
                Cow::Owned(format!("Class hierarchy for class {} forms a cycle", class))
            }
            ErrorKind::IndexEmpty => Cow::Borrowed("Cannot index empty array"),
            ErrorKind::CallLength => {
                Cow::Borrowed("Incorrect number of arguments for function call")
            }
            ErrorKind::InitLength => Cow::Borrowed("Initialization mismatch"),
            ErrorKind::InitProcedure => Cow::Borrowed("Cannot initialize with a procedure"),
            ErrorKind::Unreachable => Cow::Borrowed("Unreachable statement"),
            ErrorKind::MissingReturn => Cow::Borrowed("Missing return statement"),
            ErrorKind::ReturnMismatch => Cow::Borrowed("Return mismatch"),
            ErrorKind::NameClash => Cow::Borrowed("Name already bound in environment"),
            ErrorKind::SignatureMismatch => {
                Cow::Borrowed("Implementation does not match signature")
            }
            ErrorKind::Mismatch { expected, found } => {
                Cow::Owned(format!("Expected {} but found {}", expected, found))
            }
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} error:{}", self.span, self.kind.message())
    }
}

impl error::Report for Error {
    fn report(&self) -> ariadne::ReportBuilder<span::Span> {
        use ariadne::Span as _;
        ariadne::Report::build(
            ariadne::ReportKind::Error,
            *self.span.source(),
            self.span.lo.idx,
        )
        .with_label(ariadne::Label::new(self.span).with_message(self.kind.message()))
    }
}
