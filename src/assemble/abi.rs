#![allow(dead_code)]

//! # System V ABI
//!
//! The stack layout is as follows:
//!
//! ```text
//!  BOTTOM (lower memory address)
//!
//! |-----------------------------|
//! | return address              |
//! |-----------------------------|
//! | saved rbp                   |
//! |-----------------------------| <- rbp
//! | spilled locals              |
//! | ...                         |
//! | ...                         |
//! |-----------------------------|
//! | optional 8-byte alignment   |
//! |-----------------------------|
//! | multiple returns (3+)       |
//! | ...                         |
//! | ...                         |
//! |-----------------------------| <- address passed to callee with 3+
//! | multiple arguments (7+)     |    returns as first (implicit) argument
//! | ...                         |
//! | ...                         |
//! |-----------------------------| <- rsp (16-byte aligned)
//!
//!   TOP (lower memory address)
//! ```

use crate::constants;
use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::operand::Unary;

/// Pass `argument` to callee function.
pub fn write_argument(index: usize) -> Unary<Temporary> {
    let register = match index {
        0 => Register::Rdi,
        1 => Register::Rsi,
        2 => Register::Rdx,
        3 => Register::Rcx,
        4 => Register::R8,
        5 => Register::R9,
        _ => {
            return Unary::M(Memory::BO {
                base: Temporary::Register(Register::Rsp),
                offset: Immediate::Integer((index as i64 - 6) * constants::WORD_SIZE),
            });
        }
    };

    Unary::from(register)
}

/// Return `r#return` to calling function.
///
/// The caller will pass `address`, pointing to a stack location to write to.
pub fn write_return(address: Temporary, index: usize) -> Unary<Temporary> {
    match index {
        0 => Unary::from(Register::Rax),
        1 => Unary::from(Register::Rdx),
        _ => Unary::M(Memory::BO {
            base: address,
            offset: Immediate::Integer((index as i64 - 2) * constants::WORD_SIZE),
        }),
    }
}
