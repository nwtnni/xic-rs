use std::collections::HashMap;

use crate::data::ir;
use crate::data::lir;
use crate::data::hir;
use crate::data::operand;

#[derive(Debug, Default)]
pub struct Canonizer {
    canonized: Vec<lir::Stm>,
    purity: bool,
}

impl Canonizer {
    pub fn new() -> Self { Canonizer::default() }

    pub fn canonize_unit(mut self, unit: ir::Unit<hir::Fun>) -> ir::Unit<lir::Fun> {
        let mut funs = HashMap::default();
        for (name, fun) in unit.funs {
            funs.insert(name, self.canonize_fun(&fun));
        }
        ir::Unit {
            name: unit.name,
            funs: funs,
            data: unit.data,
        }
    }

    fn canonize_fun(&mut self, fun: &hir::Fun) -> lir::Fun {
        self.canonize_stm(&fun.body);
        let mut canonized = std::mem::replace(&mut self.canonized, Vec::new());
        if let Some(lir::Stm::Return(_)) = canonized.last() {} else {
            canonized.push(lir::Stm::Return(vec![])); 
        }
        lir::Fun {
            name: fun.name,
            body: canonized,
        }
    }

    fn canonize_exp(&mut self, exp: &hir::Exp) -> lir::Exp {
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
        }
    }

    fn canonize_stm(&mut self, stm: &hir::Stm) {
        use hir::Stm::*;
        match stm {
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
        | Call(f, args) => {
            let f = self.canonize_exp(f);            
            let i = self.canonized.len();

            let mut purity = Vec::new();
            let mut canonized = Vec::new();
            let mut indices = Vec::new();

            for arg in args {
                self.purity = true;
                canonized.push(self.canonize_exp(arg));
                indices.push(self.canonized.len());
                purity.push(self.purity);
            }

            // Find last impure argument
            if let Some(j) = purity.iter().rposition(|purity| !purity) {

                // Move previous arguments into temps
                let saved = (0..j)
                    .map(|_| operand::Temp::new("SAVE"))
                    .collect::<Vec<_>>();

                for k in (0..j).rev() {
                    let save = lir::Exp::Temp(saved[k]);
                    let into = lir::Stm::Move(save, canonized.remove(k));
                    self.canonized.insert(indices[k], into);
                }

                // Move function address into temp
                let save = lir::Exp::Temp(operand::Temp::new("SAVE"));
                let into = lir::Stm::Move(save.clone(), f);
                self.canonized.insert(i, into);

                // Collect saved temps
                let args = saved.into_iter()
                    .map(lir::Exp::Temp)
                    .chain(canonized.into_iter())
                    .collect::<Vec<_>>();

                self.canonized.push(lir::Stm::Call(save, args));
            } else {
                self.canonized.push(lir::Stm::Call(f, canonized));
            }

            self.purity = false;
        }
        | Return(exps) => {

            let mut purity = Vec::new();
            let mut canonized = Vec::new();
            let mut indices = Vec::new();

            for exp in exps {
                self.purity = true;
                canonized.push(self.canonize_exp(exp));
                indices.push(self.canonized.len());
                purity.push(self.purity);
            }

            // Find last impure argument
            if let Some(j) = purity.iter().rposition(|purity| !purity) {

                // Move previous arguments into temps
                let saved = (0..j)
                    .map(|_| operand::Temp::new("SAVE"))
                    .collect::<Vec<_>>();

                for k in (0..j).rev() {
                    let save = lir::Exp::Temp(saved[k]);
                    let into = lir::Stm::Move(save, canonized.remove(k));
                    self.canonized.insert(indices[k], into);
                }

                // Collect saved temps
                let exps = saved.into_iter()
                    .map(lir::Exp::Temp)
                    .chain(canonized.into_iter())
                    .collect::<Vec<_>>();

                self.canonized.push(lir::Stm::Return(exps));
            } else {
                self.canonized.push(lir::Stm::Return(canonized));
            }

            // Does this matter?
            self.purity = true;
        }
        }
    }

}
