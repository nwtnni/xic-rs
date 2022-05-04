use std::fmt;

use crate::abi;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::symbol;

pub struct Intel<T>(pub T);

impl<T: fmt::Display> fmt::Display for Intel<&asm::Unit<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "{}\n", asm::Directive::Intel)?;
        writeln!(fmt, "{}\n", asm::Directive::Data)?;

        for (symbol, label) in &self.0.data {
            let string = symbol::resolve(*symbol);

            writeln!(fmt, "{}", asm::Directive::Local(*label))?;
            writeln!(fmt, "{}", asm::Directive::Align(abi::WORD as usize))?;
            writeln!(fmt, "{}", asm::Directive::Quad(vec![string.len() as i64]))?;
            writeln!(fmt, "{}", Intel(&asm::Assembly::<T>::Label(*label)))?;
            writeln!(
                fmt,
                "{}\n",
                asm::Directive::Quad(string.bytes().map(|byte| byte as i64).collect())
            )?;
        }

        writeln!(fmt, "{}\n", asm::Directive::Text)?;

        for (name, function) in &self.0.functions {
            writeln!(fmt, "{}", asm::Directive::Global(Label::Fixed(*name)))?;
            writeln!(fmt, "{}", Intel(function))?;
        }

        Ok(())
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&asm::Function<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "{}:", self.0.name)?;
        for instruction in &self.0.instructions {
            if !matches!(instruction, asm::Assembly::Label(_)) {
                write!(fmt, "  ")?;
            }

            writeln!(fmt, "{}", Intel(instruction))?;
        }
        Ok(())
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&asm::Assembly<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            asm::Assembly::Binary(binary, operands) => {
                write!(fmt, "{} {}", binary, Intel(operands))
            }
            asm::Assembly::Unary(unary, operand) => write!(fmt, "{} {}", unary, Intel(operand)),
            asm::Assembly::Nullary(nullary) => write!(fmt, "{}", nullary),
            asm::Assembly::Label(label) => write!(fmt, "{}:", label),
            asm::Assembly::Jmp(label) => write!(fmt, "jmp {}", label),
            asm::Assembly::Jcc(condition, label) => write!(fmt, "j{} {}", condition, label),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&operand::Binary<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            operand::Binary::RI {
                destination,
                source: source @ Immediate::Integer(_),
            } => write!(fmt, "{}, {}", destination, source),
            operand::Binary::RI {
                destination,
                source: source @ Immediate::Label(_),
            } => write!(fmt, "{}, offset {}", destination, source),
            operand::Binary::MI {
                destination,
                source: source @ Immediate::Integer(_),
            } => write!(fmt, "qword ptr {}, {}", Intel(destination), source),
            operand::Binary::MI {
                destination,
                source: source @ Immediate::Label(_),
            } => write!(fmt, "qword ptr {}, offset {}", Intel(destination), source),
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
            operand::Unary::M(memory) => write!(fmt, "qword ptr {}", Intel(memory)),
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
                write!(fmt, "[{} + {} * {}]", base, index, scale)
            }
            operand::Memory::ISO {
                index,
                scale,
                offset,
            } => {
                write!(fmt, "[{} * {} + {}]", index, scale, offset)
            }
            operand::Memory::BISO {
                base,
                index,
                scale,
                offset,
            } => {
                write!(fmt, "[{} + {} * {} + {}]", base, index, scale, offset)
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
            asm::Unary::Call {
                arguments: _,
                returns: _,
            } => "call",
            asm::Unary::Mul | asm::Unary::Hul => "imul",
            asm::Unary::Div | asm::Unary::Mod => "idiv",
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
            asm::Nullary::Ret(_) => "ret",
        };

        write!(fmt, "{}", nullary)
    }
}
