//! # System V ABI
//!
//! The stack layout is as follows:
//!
//! ```text
//! BOTTOM (higher memory address)
//!
//! |-----------------------------|
//! | return address              |
//! |-----------------------------| <- old rsp
//! | optional 8-byte alignment   |
//! |-----------------------------|
//! | callee-saved registers      |
//! | ...                         |
//! | ...                         |
//! |-----------------------------|
//! | spilled locals              |
//! | ...                         |
//! | ...                         |
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

pub mod class;
pub mod mangle;

use crate::data::operand::Immediate;
use crate::data::operand::Memory;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::operand::Unary;

pub const WORD: i64 = 8;

pub const XI_INIT_GLOBALS: &str = "_Iinit_globals";
pub const XI_INIT_CLASSES: &str = "_Iinit_classes";
pub const XI_MAIN: &str = "_Imain_paai";
pub const XI_ALLOC: &str = "_xi_alloc";
pub const XI_OUT_OF_BOUNDS: &str = "_xi_out_of_bounds";
pub const XI_CONCAT: &str = "_xi_concat";
pub const XI_MEMDUP: &str = "_xi_memdup";
pub const XI_PRINT: &str = "_Iprint_pai";
pub const XI_PRINTLN: &str = "_Iprintln_pai";
pub const XI_READLN: &str = "_Ireadln_ai";
pub const XI_GETCHAR: &str = "_Igetchar_i";
pub const XI_EOF: &str = "_Ieof_b";
pub const XI_UNPARSE_INT: &str = "_IunparseInt_aii";
pub const XI_PARSE_INT: &str = "_IparseInt_t2ibai";
pub const XI_ASSERT: &str = "_Iassert_pb";

pub const CALLEE_SAVED: &[Register] = &[
    Register::rsp(),
    Register::Rbx,
    Register::Rbp,
    Register::R12,
    Register::R13,
    Register::R14,
    Register::R15,
];

pub const CALLER_SAVED: &[Register] = &[
    Register::Rax,
    Register::Rcx,
    Register::Rdx,
    Register::Rsi,
    Register::Rdi,
    Register::R8,
    Register::R9,
    Register::R10,
    Register::R11,
];

pub const ARGUMENT: &[Register] = &[
    Register::Rdi,
    Register::Rsi,
    Register::Rdx,
    Register::Rcx,
    Register::R8,
    Register::R9,
];

pub const RETURN: &[Register] = &[Register::Rax, Register::Rdx];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Abi {
    /// Conform to the Xi++ specification for class layouts.
    Xi,

    /// Omit virtual table or virtual table entries for final classes.
    XiFinal,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FramePointer {
    /// Output a function prologue and epilogue to preserve the frame pointer for debugging.
    Keep,

    /// Skip the function prologue and epilogue and allow the frame pointer to be
    /// used as a general-purpose register.
    Omit,
}

/// Total stack size. Guaranteed to align to 16 bytes if there is a function call.
pub fn stack_size(
    frame_pointer: FramePointer,
    callee_arguments: Option<usize>,
    callee_returns: Option<usize>,
    spilled: usize,
) -> usize {
    let unaligned = callee_arguments.unwrap_or(0).saturating_sub(6)
        + callee_returns.unwrap_or(0).saturating_sub(2)
        + spilled;

    // The stack must be aligned to 16 bytes if we need to `call` a sub-function.
    //
    // It starts off unaligned because the caller's `call` statement pushes `rip`
    // onto the stack, but if we keep the frame pointer, then it is once again aligned.
    let aligned = match frame_pointer {
        _ if callee_arguments.is_none() && callee_returns.is_none() => unaligned,
        FramePointer::Keep => (unaligned + 1) & !1,
        FramePointer::Omit => unaligned | 1,
    };

    aligned * WORD as usize
}

/// Offset of spilled temporary `index` from the stack pointer.
pub fn stack_offset(
    callee_arguments: Option<usize>,
    callee_returns: Option<usize>,
    index: usize,
) -> usize {
    WORD as usize
        * (callee_arguments.unwrap_or(0).saturating_sub(6)
            + callee_returns.unwrap_or(0).saturating_sub(2)
            + index)
}

/// Retrieve `argument` from calling function.
///
/// Extra arguments are stored in the caller's stack frame.
pub fn read_argument(index: usize) -> Unary<Temporary> {
    if let Some(register) = ARGUMENT.get(index) {
        return Unary::from(*register);
    }

    Unary::M(Memory::BO {
        base: Temporary::Register(Register::rsp_placeholder()),
        offset: Immediate::Integer((1 /* rip */ + index as i64 - 6) * WORD),
    })
}

/// Pass `argument` to callee function.
pub fn write_argument(index: usize) -> Unary<Temporary> {
    if let Some(register) = ARGUMENT.get(index) {
        return Unary::from(*register);
    }

    Unary::M(Memory::BO {
        base: Temporary::Register(Register::rsp()),
        offset: Immediate::Integer((index as i64 - 6) * WORD),
    })
}

/// Retrieve `r#return` from callee function.
///
/// Multiple returns are stored below multiple arguments (if any) in the stack.
pub fn read_return(arguments: usize, index: usize) -> Unary<Temporary> {
    if let Some(register) = RETURN.get(index) {
        return Unary::from(*register);
    }

    Unary::M(Memory::BO {
        base: Temporary::Register(Register::rsp()),
        offset: Immediate::Integer((arguments.saturating_sub(6) + index - 2) as i64 * WORD),
    })
}

/// Return `r#return` to calling function.
///
/// The caller will pass `address`, pointing to a stack location to write to.
pub fn write_return(address: Option<Temporary>, index: usize) -> Unary<Temporary> {
    if let Some(register) = RETURN.get(index) {
        return Unary::from(*register);
    }

    Unary::M(Memory::BO {
        base: address.expect("[INTERNAL ERROR]: missing return address"),
        offset: Immediate::Integer((index as i64 - 2) * WORD),
    })
}
