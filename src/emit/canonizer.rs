use crate::data::ir;
use crate::data::lir;
use crate::data::hir;

pub struct Canonizer {
    canonized: Vec<lir::Stm>,
}

impl Canonizer {

    pub fn canonize_unit(self, unit: &ir::Unit<hir::Fun>) -> ir::Unit<lir::Fun> {
        unimplemented!()
    }

    pub fn canonize_exp(&mut self, exp: &hir::Exp) -> lir::Exp {
        use hir::Exp::*;
        match exp {
        | Int(i) => lir::Exp::Int(*i),
        | Name(l) => lir::Exp::Name(*l),
        | Temp(t) => lir::Exp::Temp(*t),
        | _ => unimplemented!(),
        }
    }

    pub fn canonize_stm(&mut self, stm: &hir::Stm) -> lir::Stm {
        unimplemented!()
    }

}
