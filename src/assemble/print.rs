use std::fmt;

use crate::abi;
use crate::data::asm;
use crate::data::asm::Directive;
use crate::data::asm::Statement;
use crate::data::ir;
use crate::data::operand::Binary;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Memory;
use crate::data::operand::Unary;

pub struct Intel<T>(pub T);

impl<T: fmt::Display> fmt::Display for Intel<&asm::Unit<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "{}\n", Directive::Intel)?;

        writeln!(fmt, "{}\n", Directive::Data)?;

        for (symbol, data) in &self.0.data {
            writeln!(
                fmt,
                "{}",
                Directive::Visible(ir::Visibility::Local, Label::Fixed(*symbol))
            )?;
            writeln!(fmt, "{}", Directive::Align(abi::WORD as usize))?;
            writeln!(
                fmt,
                "{}",
                Intel(&Statement::<T>::Label(Label::Fixed(*symbol)))
            )?;
            writeln!(fmt, "{}\n", Directive::Quad(data.clone()))?;
        }

        writeln!(fmt, "{}\n", Directive::Bss)?;

        for (symbol, (visibility, size)) in &self.0.bss {
            writeln!(
                fmt,
                "{}",
                Directive::Visible(*visibility, Label::Fixed(*symbol))
            )?;
            writeln!(fmt, "{}", Directive::Align(abi::WORD as usize))?;
            writeln!(
                fmt,
                "{}",
                Intel(&Statement::<T>::Label(Label::Fixed(*symbol))),
            )?;
            writeln!(fmt, "{}\n", Directive::Space(*size * abi::WORD as usize),)?;
        }

        writeln!(fmt, "{}", Directive::Ctors)?;
        writeln!(fmt, "{}", Directive::Align(abi::WORD as usize))?;
        writeln!(
            fmt,
            "{}\n",
            Directive::Quad(vec![Immediate::Label(Label::Fixed(abi::mangle::init()))])
        )?;

        writeln!(fmt, "{}\n", Directive::Text)?;

        for (name, function) in &self.0.functions {
            writeln!(
                fmt,
                "{}",
                Directive::Visible(function.visibility, Label::Fixed(*name))
            )?;
            writeln!(fmt, "{}", Intel(function))?;
        }

        Ok(())
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&asm::Function<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "{}:", self.0.name)?;
        for statement in &self.0.statements {
            if !matches!(statement, Statement::Label(_)) {
                write!(fmt, "  ")?;
            }

            writeln!(fmt, "{}", Intel(statement))?;
        }
        Ok(())
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&Statement<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Statement::Binary(binary, operands) => {
                write!(fmt, "{} {}", binary, Intel(operands))
            }
            Statement::Unary(unary, operand) => write!(fmt, "{} {}", unary, Intel(operand)),
            Statement::Nullary(nullary) => write!(fmt, "{}", nullary),
            Statement::Label(label) => write!(fmt, "{}:", label),
            Statement::Jmp(label) => write!(fmt, "jmp {}", label),
            Statement::Jcc(condition, label) => write!(fmt, "j{} {}", condition, label),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&Binary<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Binary::RI {
                destination,
                source: source @ Immediate::Integer(_),
            } => write!(fmt, "{}, {}", destination, source),
            Binary::RI {
                destination,
                source: source @ Immediate::Label(_),
            } => write!(fmt, "{}, offset {}", destination, source),
            Binary::MI {
                destination,
                source: source @ Immediate::Integer(_),
            } => write!(fmt, "qword ptr {}, {}", Intel(destination), source),
            Binary::MI {
                destination,
                source: source @ Immediate::Label(_),
            } => write!(fmt, "qword ptr {}, offset {}", Intel(destination), source),
            Binary::MR {
                destination,
                source,
            } => write!(fmt, "{}, {}", Intel(destination), source),
            Binary::RM {
                destination,
                source,
            } => write!(fmt, "{}, {}", destination, Intel(source)),
            Binary::RR {
                destination,
                source,
            } => write!(fmt, "{}, {}", destination, source),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&Unary<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Unary::I(immediate) => write!(fmt, "{}", immediate),
            Unary::R(register) => write!(fmt, "{}", register),
            Unary::M(memory) => write!(fmt, "qword ptr {}", Intel(memory)),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Intel<&Memory<T>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Memory::B { base } => write!(fmt, "[{}]", base),
            Memory::O { offset } => write!(fmt, "[{}]", offset),
            Memory::BI { base, index } => {
                write!(fmt, "[{} + {}]", base, index)
            }
            Memory::BO { base, offset } => {
                write!(fmt, "[{} + {}]", base, offset)
            }
            Memory::BIO {
                base,
                index,
                offset,
            } => {
                write!(fmt, "[{} + {} + {}]", base, index, offset)
            }
            Memory::BIS { base, index, scale } => {
                write!(fmt, "[{} + {} * {}]", base, index, scale)
            }
            Memory::ISO {
                index,
                scale,
                offset,
            } => {
                write!(fmt, "[{} * {} + {}]", index, scale, offset)
            }
            Memory::BISO {
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

impl fmt::Display for Directive {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Directive::Intel => write!(fmt, ".intel_syntax noprefix"),
            Directive::Align(alignment) => write!(fmt, ".align {}", alignment),
            Directive::Visible(ir::Visibility::Local, label) => {
                write!(fmt, ".local {}", label)
            }
            Directive::Visible(ir::Visibility::Global, label) => {
                write!(fmt, ".global {}", label)
            }
            Directive::Quad(data) => {
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
            Directive::Space(bytes) => write!(fmt, ".space {}", bytes),
            Directive::Data => write!(fmt, ".section .data"),
            Directive::Bss => write!(fmt, ".section .bss"),
            Directive::Ctors => write!(fmt, ".section .ctors"),
            Directive::Text => write!(fmt, ".section .text"),
        }
    }
}

impl fmt::Display for asm::Binary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let binary = match self {
            asm::Binary::Add => "add",
            asm::Binary::Sub => "sub",
            asm::Binary::And => "and",
            asm::Binary::Shl => "shl",
            asm::Binary::Mul => "imul",
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
            asm::Unary::Hul => "imul",
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
            asm::Condition::Ae => "ae",
        };

        write!(fmt, "{}", condition)
    }
}

impl fmt::Display for asm::Nullary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let nullary = match self {
            asm::Nullary::Cqo => "cqo",
            asm::Nullary::Ret(_) => "ret",
            asm::Nullary::Nop => "nop",
        };

        write!(fmt, "{}", nullary)
    }
}
