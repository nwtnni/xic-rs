use std::borrow::Cow;

use crate::data::r#type;
use crate::data::span::Span;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::error;

#[derive(Clone, Debug)]
pub struct Error {
    span: Span,
    // Tracks the chain of template instantiations to where a type error occurs.
    provenance: Vec<Span>,
    kind: ErrorKind,
}

impl Error {
    pub fn new(span: Span, kind: ErrorKind) -> Self {
        Error {
            span,
            provenance: Vec::new(),
            kind,
        }
    }

    pub(super) fn with_provenance(mut self, provenance: Vec<Span>) -> Self {
        self.provenance = provenance;
        self
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
    UnboundClass(Symbol),
    UnboundClassTemplate(Symbol),
    NotVariable(Symbol),
    NotFun(Option<Symbol>),
    NotExp,
    NotProcedure,
    NotClass,
    NotInClass(Option<Symbol>),
    NotInClassModule(Symbol),
    NotInWhile,
    NoSuperclass(Symbol),
    ClassCycle(Symbol),
    ClassIncomplete(Symbol, Span),
    IndexEmpty,
    CallLength,
    InitLength,
    InitProcedure,
    Unreachable,
    MissingReturn,
    ReturnMismatch,
    NameClash(Span),
    SignatureMismatch(Span),
    TemplateArgumentMismatch {
        span: Span,
        expected: usize,
        found: usize,
    },
    Mismatch {
        expected: r#type::Expression,
        expected_span: Option<Span>,
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
            ErrorKind::UnboundClass(c) => {
                Cow::Owned(format!("Unbound class {}", symbol::resolve(*c)))
            }
            ErrorKind::UnboundClassTemplate(c) => {
                Cow::Owned(format!("Unbound class template {}", symbol::resolve(*c)))
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
            ErrorKind::NotInWhile => Cow::Borrowed("Not inside while loop"),
            ErrorKind::NoSuperclass(class) => {
                Cow::Owned(format!("Class {} has no superclass", class))
            }
            ErrorKind::ClassCycle(class) => {
                Cow::Owned(format!("Class hierarchy for class {} forms a cycle", class))
            }
            ErrorKind::ClassIncomplete(class, _) => Cow::Owned(format!(
                "Class {} does not implement method required in interface",
                class
            )),
            ErrorKind::IndexEmpty => Cow::Borrowed("Cannot index empty array"),
            ErrorKind::CallLength => {
                Cow::Borrowed("Incorrect number of arguments for function call")
            }
            ErrorKind::InitLength => Cow::Borrowed("Initialization mismatch"),
            ErrorKind::InitProcedure => Cow::Borrowed("Cannot initialize with a procedure"),
            ErrorKind::Unreachable => Cow::Borrowed("Unreachable statement"),
            ErrorKind::MissingReturn => Cow::Borrowed("Missing return statement"),
            ErrorKind::ReturnMismatch => Cow::Borrowed("Return mismatch"),
            ErrorKind::NameClash(_) => Cow::Borrowed("Name already bound in environment"),
            ErrorKind::SignatureMismatch(_) => {
                Cow::Borrowed("Implementation does not match signature")
            }
            ErrorKind::TemplateArgumentMismatch { span: _, expected, found } => {
                Cow::Owned(format!(
                    "Template instantiated with incorrect number of type arguments: expected {}, but found {}",
                    expected,
                    found
                ))
            }
            ErrorKind::Mismatch {
                expected,
                expected_span: _,
                found,
            } => Cow::Owned(format!("Expected {} but found {}", expected, found)),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} error:{}", self.span, self.kind.message())
    }
}

impl error::Report for Error {
    fn report(&self) -> ariadne::ReportBuilder<Span> {
        use ariadne::Span as _;
        let report = ariadne::Report::build(
            ariadne::ReportKind::Error,
            *self.span.source(),
            self.span.lo.index(),
        )
        .with_label(ariadne::Label::new(self.span).with_message(self.kind.message()));

        let mut report = match &self.kind {
            ErrorKind::Mismatch {
                expected,
                expected_span: Some(span),
                found: _,
            } => report.with_label(
                ariadne::Label::new(*span)
                    .with_message(format!("Expected {} because of this", expected)),
            ),
            ErrorKind::ClassIncomplete(_, span) => {
                report.with_label(ariadne::Label::new(*span).with_message("Method required here"))
            }
            ErrorKind::NameClash(span) => report
                .with_label(ariadne::Label::new(*span).with_message("Previous definition here")),
            ErrorKind::SignatureMismatch(span) => report
                .with_label(ariadne::Label::new(*span).with_message("Signature definition here")),
            ErrorKind::TemplateArgumentMismatch { span, .. } => report
                .with_label(ariadne::Label::new(*span).with_message("Template definition here")),
            _ => report,
        };

        report.add_labels(
            self.provenance
                .iter()
                .rev()
                .map(|span| ariadne::Label::new(*span).with_message("Template instantiated here")),
        );

        report
    }
}
