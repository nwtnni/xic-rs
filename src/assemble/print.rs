use std::fmt;

use crate::data::asm;
use crate::data::operand;

pub struct Intel<T>(T);

impl<T: fmt::Display> fmt::Display for Intel<&asm::Assembly<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            asm::Assembly::Binary(binary, operands) => {
                write!(fmt, "    {} {}", binary, Intel(operands))
            }
            asm::Assembly::Unary(unary, operand) => write!(fmt, "    {} {}", unary, Intel(operand)),
            asm::Assembly::Nullary(nullary) => write!(fmt, "    {}", nullary),
            asm::Assembly::Label(label) => write!(fmt, "{}:", label),
            asm::Assembly::Directive(directive) => write!(fmt, "{}", directive),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&operand::Binary<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            operand::Binary::RI {
                destination,
                source,
            } => write!(fmt, "{}, {}", destination, source),
            operand::Binary::MI {
                destination,
                source,
            } => write!(fmt, "{}, {}", Intel(destination), source),
            operand::Binary::MR {
                destination,
                source,
            } => write!(fmt, "{}, {}", Intel(destination), source),
            operand::Binary::RM {
                destination,
                source,
            } => write!(fmt, "{}, {}", destination, Intel(source)),
            operand::Binary::RR {
                destination,
                source,
            } => write!(fmt, "{}, {}", destination, source),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&operand::Unary<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            operand::Unary::I(immediate) => write!(fmt, "{}", immediate),
            operand::Unary::R(register) => write!(fmt, "{}", register),
            operand::Unary::M(memory) => write!(fmt, "{}", Intel(memory)),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&operand::Memory<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            operand::Memory::B { base } => write!(fmt, "[{}]", base),
            operand::Memory::O { offset } => write!(fmt, "[{}]", offset),
            operand::Memory::BI { base, index } => {
                write!(fmt, "[{} + {}]", base, index)
            }
            operand::Memory::BO { base, offset } => {
                write!(fmt, "[{} + {}]", base, offset)
            }
            operand::Memory::BIO {
                base,
                index,
                offset,
            } => {
                write!(fmt, "[{} + {} + {}]", base, index, offset)
            }
            operand::Memory::BIS { base, index, scale } => {
                write!(fmt, "[{} + {}*{}]", base, scale, index)
            }
            operand::Memory::ISO {
                index,
                scale,
                offset,
            } => {
                write!(fmt, "[{}*{} + {}]", index, scale, offset)
            }
            operand::Memory::BISO {
                base,
                index,
                scale,
                offset,
            } => {
                write!(fmt, "[{} + {}*{} + {}]", base, index, scale, offset)
            }
        }
    }
}

impl fmt::Display for asm::Directive {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            asm::Directive::Intel => write!(fmt, ".intel_syntax noprefix"),
            asm::Directive::Align(alignment) => write!(fmt, ".align {}", alignment),
            asm::Directive::Local(label) => write!(fmt, ".local {}", label),
            asm::Directive::Global(label) => write!(fmt, ".global {}", label),
            asm::Directive::Quad(data) => {
                write!(fmt, ".quad")?;

                let mut iter = data.iter();

                if let Some(head) = iter.next() {
                    write!(fmt, " {}", head)?;
                }

                for tail in iter {
                    write!(fmt, ", {}", tail)?;
                }

                Ok(())
            }
            asm::Directive::Data => write!(fmt, ".data"),
            asm::Directive::Text => write!(fmt, ".text"),
        }
    }
}

impl fmt::Display for asm::Binary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let binary = match self {
            asm::Binary::Add => "add",
            asm::Binary::Sub => "sub",
            asm::Binary::And => "and",
            asm::Binary::Or => "or",
            asm::Binary::Xor => "xor",
            asm::Binary::Cmp => "cmp",
            asm::Binary::Mov => "mov",
            asm::Binary::Lea => "lea",
        };

        write!(fmt, "{}", binary)
    }
}

impl fmt::Display for asm::Unary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let unary = match self {
            asm::Unary::Neg => "neg",
            asm::Unary::Push => "push",
            asm::Unary::Pop => "pop",
            asm::Unary::Call {
                arguments: _,
                returns: _,
            } => "call",
            asm::Unary::Mul => "imul",
            asm::Unary::Div(_) => "idiv",
            asm::Unary::Jmp => "jmp",
            asm::Unary::Jcc(condition) => {
                return write!(fmt, "j{}", condition);
            }
        };

        write!(fmt, "{}", unary)
    }
}

impl fmt::Display for asm::Condition {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let condition = match self {
            asm::Condition::E => "e",
            asm::Condition::Ne => "ne",
            asm::Condition::G => "g",
            asm::Condition::Ge => "ge",
            asm::Condition::L => "l",
            asm::Condition::Le => "le",
        };

        write!(fmt, "{}", condition)
    }
}

impl fmt::Display for asm::Nullary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let nullary = match self {
            asm::Nullary::Cqo => "cqo",
            asm::Nullary::Ret => "ret",
        };

        write!(fmt, "{}", nullary)
    }
}
