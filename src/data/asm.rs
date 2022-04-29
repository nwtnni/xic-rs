use crate::data::ir;
use crate::data::operand;

pub type Unit<T> = ir::Unit<Function<T>>;
pub type Function<T> = ir::Function<Vec<Assembly<T>>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Assembly<T> {
    Binary(Binary, operand::Binary<T>),
    Unary(Unary, operand::Unary<T>),
    Nullary(Nullary),
    Label(operand::Label),
    Directive(Directive),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Directive {
    Intel,
    Align(usize),
    Local(operand::Label),
    Global(operand::Label),
    Quad(Vec<i64>),
    Data,
    Text,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Binary {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Cmp,
    Mov,
    Lea,
}

impl From<ir::Binary> for Binary {
    fn from(binary: ir::Binary) -> Self {
        match binary {
            ir::Binary::Add => Binary::Add,
            ir::Binary::Sub => Binary::Sub,
            ir::Binary::Xor => Binary::Xor,
            ir::Binary::And => Binary::And,
            ir::Binary::Or => Binary::Or,
            _ => panic!("[INTERNAL ERROR]: converting unsupported IR operator"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unary {
    Neg,
    Push,
    Pop,
    Call { arguments: usize, returns: usize },
    Mul,
    Div(Division),
    Jmp,
    Jcc(Condition),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nullary {
    Cqo,
    Ret,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    L,
    Le,
    Ge,
    G,
    Ne,
    E,
}

impl From<ir::Condition> for Condition {
    fn from(condition: ir::Condition) -> Self {
        match condition {
            ir::Condition::Lt => Condition::L,
            ir::Condition::Le => Condition::Le,
            ir::Condition::Ge => Condition::Ge,
            ir::Condition::Gt => Condition::G,
            ir::Condition::Ne => Condition::Ne,
            ir::Condition::Eq => Condition::E,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Division {
    Quotient,
    Remainder,
}
