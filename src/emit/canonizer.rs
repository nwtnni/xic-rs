use crate::data::ir;
use crate::data::lir;
use crate::data::hir;
use crate::data::operand;

pub struct Canonizer {
    canonized: Vec<lir::Stm>,
    purity: bool,
}

impl Canonizer {

    pub fn canonize_unit(self, unit: &ir::Unit<hir::Fun>) -> ir::Unit<lir::Fun> {
        unimplemented!()
    }

    pub fn canonize_exp(&mut self, exp: &hir::Exp) -> lir::Exp {
        use hir::Exp::*;
        match exp {
        | Int(i) => lir::Exp::Int(*i),
        | Mem(e) => self.canonize_exp(e),
        | Name(l) => lir::Exp::Name(*l),
        | Temp(t) => lir::Exp::Temp(*t),
        | ESeq(s, e) => {
            self.canonize_stm(s);
            self.canonize_exp(e)
        }
        | Bin(b, l, r) => {
            let l = self.canonize_exp(l);
            let i = self.canonized.len();
            let r = self.canonize_exp(r);
            if self.purity {
                lir::Exp::Bin(*b, Box::new(l), Box::new(r))
            } else {
                let save = lir::Exp::Temp(operand::Temp::new("SAVE"));
                let into = lir::Stm::Move(save.clone(), l);
                self.canonized.insert(i, into);
                lir::Exp::Bin(*b, Box::new(save), Box::new(r))
            }
        }
        | _ => unimplemented!(),
        }
    }

    pub fn canonize_stm(&mut self, stm: &hir::Stm) {
        use hir::Stm::*;
        match stm {
        | Exp(e) => { self.canonize_exp(e); },
        | Label(l) => self.canonized.push(lir::Stm::Label(*l)),
        | Seq(stms) => {
            let mut purity = true;
            for stm in stms {
                self.purity = true;
                self.canonize_stm(stm);
                purity &= self.purity;
            }
            self.purity = purity;
        }
        | Jump(e) => {
            let jump = lir::Stm::Jump(self.canonize_exp(e));
            self.canonized.push(jump);
            self.purity = false;
        }
        | CJump(e, t, f) => {
            let cjump = lir::Stm::CJump(self.canonize_exp(e), *t, *f);
            self.canonized.push(cjump);
            self.purity = false;
        }
        | Move(hir::Exp::Mem(d), s) => {
            let d = self.canonize_exp(d);
            let i = self.canonized.len();
            let s = self.canonize_exp(s);
            if self.purity {
                self.canonized.push(lir::Stm::Move(lir::Exp::Mem(Box::new(d)), s));
            } else {
                let save = lir::Exp::Temp(operand::Temp::new("SAVE")); 
                let into = lir::Stm::Move(save.clone(), d);
                self.canonized.insert(i, into);
                self.canonized.push(lir::Stm::Move(lir::Exp::Mem(Box::new(save)), s)); 
            }
            self.purity = false;

        }
        | Move(d, s) => {
            let d = self.canonize_exp(d);
            let i = self.canonized.len();
            let s = self.canonize_exp(s);
            if self.purity {
                self.canonized.push(lir::Stm::Move(d, s));
            } else {
                let save = lir::Exp::Temp(operand::Temp::new("SAVE")); 
                let into = lir::Stm::Move(save.clone(), d);
                self.canonized.insert(i, into);
                self.canonized.push(lir::Stm::Move(save, s)); 
            }
            self.purity = false;
        }
        | _ => unreachable!(),
        }
    }

}
