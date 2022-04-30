mod allocate;
mod print;
mod tile;

use crate::data::asm;
use crate::data::lir;
use crate::data::operand::Register;
use crate::data::operand::Temporary;

pub(crate) use print::Intel;

pub fn tile(lir: &lir::Unit<lir::Fallthrough>) -> asm::Unit<Temporary> {
    tile::tile_unit(lir)
}

pub fn allocate(assembly: &asm::Unit<Temporary>) -> asm::Unit<Register> {
    allocate::allocate_unit(assembly)
}
