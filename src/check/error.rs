use crate::util::span;
use crate::util::symbol;
use crate::data::typ;

#[derive(Clone, Debug)]
pub struct Error {
    span: span::Span,
    kind: ErrorKind,
}

impl Error {
    pub fn new(span: span::Span, kind: ErrorKind) -> Self {
        Error { span, kind }
    }
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnboundVar(symbol::Symbol),
    UnboundFun(symbol::Symbol),
    NotVar(symbol::Symbol),
    NotFun(symbol::Symbol),
    NotExp,
    IndexEmpty,
    CallLength,
    InitLength,
    InitProcedure,
    Unreachable,
    MissingReturn,
    NameClash,
    SigMismatch,
    Mismatch {
        expected: typ::Typ,
        found: typ::Typ,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let error = match &self.kind {
        | ErrorKind::UnboundVar(v) => format!("Unbound variable {}", symbol::resolve(*v)),
        | ErrorKind::UnboundFun(f) => format!("Unbound function {}", symbol::resolve(*f)),
        | ErrorKind::NotVar(v) => format!("{} is not a variable type", symbol::resolve(*v)),
        | ErrorKind::NotFun(f) => format!("{} is not a function", symbol::resolve(*f)),
        | ErrorKind::NotExp => format!("Not an expression type"),
        | ErrorKind::IndexEmpty => format!("Cannot index empty array"),
        | ErrorKind::CallLength => format!("Incorrect number of arguments for function call"),
        | ErrorKind::InitLength => format!("Initialization mismatch"),
        | ErrorKind::InitProcedure => format!("Cannot initialize with a procedure"),
        | ErrorKind::Unreachable => format!("Unreachable statement"),
        | ErrorKind::MissingReturn => format!("Missing return statement"),
        | ErrorKind::NameClash => format!("Name already bound in environment"),
        | ErrorKind::SigMismatch => format!("Implementation does not match signature"),
        | ErrorKind::Mismatch { expected, found } => format!("Expected {} but found {}", expected, found),
        };
        write!(fmt, "{} error:{}", self.span, error)
    }
}
