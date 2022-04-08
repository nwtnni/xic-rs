use std::collections::HashMap;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;

#[derive(Debug, Default)]
pub struct Canonizer {
    canonized: Vec<lir::Statement>,
    purity: bool,
}

impl Canonizer {
    pub fn new() -> Self {
        Canonizer::default()
    }

    pub fn canonize_unit(mut self, unit: ir::Unit<hir::Fun>) -> ir::Unit<lir::Function> {
        let mut funs = HashMap::default();
        for (name, fun) in unit.funs {
            funs.insert(name, self.canonize_fun(&fun));
        }
        ir::Unit {
            name: unit.name,
            funs,
            data: unit.data,
        }
    }

    fn canonize_fun(&mut self, fun: &hir::Fun) -> lir::Function {
        self.canonize_stm(&fun.body);
        let mut canonized = std::mem::take(&mut self.canonized);
        if let Some(lir::Statement::Return(_)) = canonized.last() {
        } else {
            canonized.push(lir::Statement::Return(vec![]));
        }
        lir::Function {
            name: fun.name,
            body: canonized,
        }
    }

    fn canonize_exp(&mut self, exp: &hir::Exp) -> lir::Expression {
        use hir::Exp::*;
        match exp {
            Int(i) => lir::Expression::Int(*i),
            Mem(e) => self.canonize_exp(e),
            Name(l) => lir::Expression::Name(*l),
            Temp(t) => lir::Expression::Temp(*t),
            ESeq(s, e) => {
                self.canonize_stm(s);
                self.canonize_exp(e)
            }
            Bin(b, l, r) => {
                let l = self.canonize_exp(l);
                let i = self.canonized.len();
                let r = self.canonize_exp(r);
                if self.purity {
                    lir::Expression::Bin(*b, Box::new(l), Box::new(r))
                } else {
                    let save = lir::Expression::Temp(operand::Temp::new("SAVE"));
                    let into = lir::Statement::Move(save.clone(), l);
                    self.canonized.insert(i, into);
                    lir::Expression::Bin(*b, Box::new(save), Box::new(r))
                }
            }
            Call(call) => {
                self.canonize_call(call);
                lir::Expression::Temp(operand::Temp::Ret(0))
            }
        }
    }

    fn canonize_stm(&mut self, stm: &hir::Stm) {
        use hir::Stm::*;
        match stm {
            Label(l) => self.canonized.push(lir::Statement::Label(*l)),
            Seq(stms) => {
                let mut purity = true;
                for stm in stms {
                    self.purity = true;
                    self.canonize_stm(stm);
                    purity &= self.purity;
                }
                self.purity = purity;
            }
            Jump(e) => {
                let jump = lir::Statement::Jump(self.canonize_exp(e));
                self.canonized.push(jump);
                self.purity = false;
            }
            CJump(e, t, f) => {
                let cjump = lir::Statement::CJump(self.canonize_exp(e), *t, *f);
                self.canonized.push(cjump);
                self.purity = false;
            }
            Move(hir::Exp::Mem(d), s) => {
                let d = self.canonize_exp(d);
                let i = self.canonized.len();
                let s = self.canonize_exp(s);
                if self.purity {
                    self.canonized
                        .push(lir::Statement::Move(lir::Expression::Mem(Box::new(d)), s));
                } else {
                    let save = lir::Expression::Temp(operand::Temp::new("SAVE"));
                    let into = lir::Statement::Move(save.clone(), d);
                    self.canonized.insert(i, into);
                    self.canonized.push(lir::Statement::Move(
                        lir::Expression::Mem(Box::new(save)),
                        s,
                    ));
                }
                self.purity = false;
            }
            Move(d, s) => {
                let d = self.canonize_exp(d);
                let i = self.canonized.len();
                let s = self.canonize_exp(s);
                if self.purity {
                    self.canonized.push(lir::Statement::Move(d, s));
                } else {
                    let save = lir::Expression::Temp(operand::Temp::new("SAVE"));
                    let into = lir::Statement::Move(save.clone(), d);
                    self.canonized.insert(i, into);
                    self.canonized.push(lir::Statement::Move(save, s));
                }
                self.purity = false;
            }
            Call(call) => self.canonize_call(call),
            Return(exps) => {
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
                        let save = lir::Expression::Temp(saved[k]);
                        let into = lir::Statement::Move(save, canonized.remove(k));
                        self.canonized.insert(indices[k], into);
                    }

                    // Collect saved temps
                    let exps = saved
                        .into_iter()
                        .map(lir::Expression::Temp)
                        .chain(canonized.into_iter())
                        .collect::<Vec<_>>();

                    self.canonized.push(lir::Statement::Return(exps));
                } else {
                    self.canonized.push(lir::Statement::Return(canonized));
                }

                // Does this matter?
                self.purity = true;
            }
        }
    }

    fn canonize_call(&mut self, call: &hir::Call) {
        let f = self.canonize_exp(&call.name);
        let i = self.canonized.len();

        let mut purity = Vec::new();
        let mut canonized = Vec::new();
        let mut indices = Vec::new();

        for arg in &call.args {
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
                let save = lir::Expression::Temp(saved[k]);
                let into = lir::Statement::Move(save, canonized.remove(k));
                self.canonized.insert(indices[k], into);
            }

            // Move function address into temp
            let save = lir::Expression::Temp(operand::Temp::new("SAVE"));
            let into = lir::Statement::Move(save.clone(), f);
            self.canonized.insert(i, into);

            // Collect saved temps
            let args = saved
                .into_iter()
                .map(lir::Expression::Temp)
                .chain(canonized.into_iter())
                .collect::<Vec<_>>();

            self.canonized.push(lir::Statement::Call(save, args));
        } else {
            self.canonized.push(lir::Statement::Call(f, canonized));
        }

        self.purity = false;
    }
}
