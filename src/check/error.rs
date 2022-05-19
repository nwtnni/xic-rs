use crate::data::r#type;
use crate::data::span;
use crate::data::symbol;
use crate::data::symbol::Symbol;

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
    UnboundVariable(Symbol),
    UnboundFun(Symbol),
    NotVariable(Symbol),
    NotFun(Symbol),
    NotExp,
    NotProcedure,
    NotInClass(Option<Symbol>),
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

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let error = match &self.kind {
            ErrorKind::UnboundVariable(v) => format!("Unbound variable {}", symbol::resolve(*v)),
            ErrorKind::UnboundFun(f) => format!("Unbound function {}", symbol::resolve(*f)),
            ErrorKind::NotVariable(v) => format!("{} is not a variable type", symbol::resolve(*v)),
            ErrorKind::NotFun(f) => format!("{} is not a function", symbol::resolve(*f)),
            ErrorKind::NotExp => String::from("Not a single expression type"),
            ErrorKind::NotProcedure => String::from("Not a procedure"),
            ErrorKind::NotInClass(None) => String::from("Not inside a class implementation"),
            ErrorKind::NotInClass(Some(class)) => format!("Not inside class {}", class),
            ErrorKind::IndexEmpty => String::from("Cannot index empty array"),
            ErrorKind::CallLength => {
                String::from("Incorrect number of arguments for function call")
            }
            ErrorKind::InitLength => String::from("Initialization mismatch"),
            ErrorKind::InitProcedure => String::from("Cannot initialize with a procedure"),
            ErrorKind::Unreachable => String::from("Unreachable statement"),
            ErrorKind::MissingReturn => String::from("Missing return statement"),
            ErrorKind::ReturnMismatch => String::from("Return mismatch"),
            ErrorKind::NameClash => String::from("Name already bound in environment"),
            ErrorKind::SignatureMismatch => String::from("Implementation does not match signature"),
            ErrorKind::Mismatch { expected, found } => {
                format!("Expected {} but found {}", expected, found)
            }
        };
        write!(fmt, "{} error:{}", self.span, error)
    }
}
