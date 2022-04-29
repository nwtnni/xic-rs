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
use crate::data::operand;

/// Pass `argument` to callee function.
pub fn callee_argument(argument: usize) -> operand::One<operand::Temporary> {
    let register = match argument {
        0 => operand::Register::Rdi,
        1 => operand::Register::Rsi,
        2 => operand::Register::Rdx,
        3 => operand::Register::Rcx,
        4 => operand::Register::R8,
        5 => operand::Register::R9,
        _ => {
            return operand::One::M(operand::Memory::BO {
                base: operand::Temporary::Register(operand::Register::Rsp),
                offset: operand::Immediate::Integer((argument as i64 - 6) * constants::WORD_SIZE),
            });
        }
    };

    operand::One::R(operand::Temporary::Register(register))
}

/// Retrieve `r#return` from callee function.
///
/// The caller will pass `address`, pointing to a stack location to write to.
pub fn callee_return(
    address: operand::Temporary,
    r#return: usize,
) -> operand::One<operand::Temporary> {
    match r#return {
        0 => operand::One::R(operand::Temporary::Register(operand::Register::Rax)),
        1 => operand::One::R(operand::Temporary::Register(operand::Register::Rdx)),
        _ => operand::One::M(operand::Memory::BO {
            base: address,
            offset: operand::Immediate::Integer((r#return as i64 - 2) * constants::WORD_SIZE),
        }),
    }
}
