mod allocate;
mod print;
mod tile;

use crate::data::asm;
use crate::data::lir;
use crate::data::operand;

pub(crate) use print::Intel;

pub fn tile(lir: &lir::Unit<lir::Fallthrough>) -> asm::Unit<operand::Temporary> {
    tile::tile_unit(lir)
}
