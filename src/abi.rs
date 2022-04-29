#![allow(dead_code, unused_variables)]

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

use std::fmt::Write as _;

use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::operand::Unary;
use crate::data::r#type;

pub const WORD: i64 = 8;

pub const XI_MAIN: &str = "_Imain_paai";
pub const XI_ALLOC: &str = "_xi_alloc";
pub const XI_OUT_OF_BOUNDS: &str = "_xi_out_of_bounds";
pub const XI_PRINT: &str = "_Iprint_pai";
pub const XI_PRINTLN: &str = "_Iprintln_pai";
pub const XI_READLN: &str = "_Ireadln_ai";
pub const XI_GETCHAR: &str = "_Igetchar_i";
pub const XI_EOF: &str = "_Ieof_b";
pub const XI_UNPARSE_INT: &str = "_IunparseInt_aii";
pub const XI_PARSE_INT: &str = "_IparseInt_t2ibai";
pub const XI_ASSERT: &str = "_Iassert_pb";

/// Retrieve `argument` from calling function.
///
/// Extra arguments are stored in the caller's stack frame.
pub fn read_argument(index: usize) -> Unary<Temporary> {
    if let Some(register) = argument_register(index) {
        return register;
    }

    Unary::M(Memory::BO {
        base: Temporary::Register(Register::Rbp),
        offset: Immediate::Integer((1 /* rbp */ + 1 /* rip */ + index as i64 - 6) * WORD),
    })
}

/// Pass `argument` to callee function.
pub fn write_argument(index: usize) -> Unary<Temporary> {
    if let Some(register) = argument_register(index) {
        return register;
    }

    Unary::M(Memory::BO {
        base: Temporary::Register(Register::Rsp),
        offset: Immediate::Integer((index as i64 - 6) * WORD),
    })
}

/// Retrieve `r#return` from callee function.
///
/// Multiple returns are stored below multiple arguments (if any) in the stack.
pub fn read_return(arguments: usize, index: usize) -> Unary<Temporary> {
    if let Some(register) = return_register(index) {
        return register;
    }

    Unary::M(Memory::BO {
        base: Temporary::Register(Register::Rsp),
        offset: Immediate::Integer((arguments.saturating_sub(6) + index - 2) as i64 * WORD),
    })
}

/// Return `r#return` to calling function.
///
/// The caller will pass `address`, pointing to a stack location to write to.
pub fn write_return(address: Option<Temporary>, index: usize) -> Unary<Temporary> {
    if let Some(register) = return_register(index) {
        return register;
    }

    Unary::M(Memory::BO {
        base: address.expect("[INTERNAL ERROR]: missing return address"),
        offset: Immediate::Integer((index as i64 - 2) * WORD),
    })
}

fn argument_register(index: usize) -> Option<Unary<Temporary>> {
    let register = match index {
        0 => Register::Rdi,
        1 => Register::Rsi,
        2 => Register::Rdx,
        3 => Register::Rcx,
        4 => Register::R8,
        5 => Register::R9,
        _ => return None,
    };

    Some(Unary::from(register))
}

fn return_register(index: usize) -> Option<Unary<Temporary>> {
    let register = match index {
        0 => Register::Rax,
        1 => Register::Rdx,
        _ => return None,
    };

    Some(Unary::from(register))
}

pub fn mangle_function(
    name: &str,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
) -> String {
    let mut mangled = format!("_I{}_", name.replace('_', "__"));

    match returns {
        [] => mangled.push('p'),
        [r#type] => {
            mangle_type(r#type, &mut mangled);
        }
        types => {
            mangled.push('t');
            write!(&mut mangled, "{}", types.len()).unwrap();
            for r#type in types {
                mangle_type(r#type, &mut mangled);
            }
        }
    }

    for parameter in parameters {
        mangle_type(parameter, &mut mangled);
    }

    mangled
}

fn mangle_type(r#type: &r#type::Expression, mangled: &mut String) {
    match r#type {
        r#type::Expression::Any => panic!("[INTERNAL ERROR]: any type in IR"),
        r#type::Expression::Integer => mangled.push('i'),
        r#type::Expression::Boolean => mangled.push('b'),
        r#type::Expression::Array(r#type) => {
            mangled.push('a');
            mangle_type(&*r#type, mangled);
        }
    }
}
