use std::collections::HashMap;

use crate::check;
use crate::data::ast;
use crate::data::ir;
use crate::data::hir;
use crate::data::operand;
use crate::util::symbol;

#[derive(Debug)]
pub struct Emitter {
    env: check::Env,
    data: HashMap<symbol::Symbol, operand::Label>,
    vars: HashMap<symbol::Symbol, operand::Temp>,
}

const XI_ALLOC: &'static str = "_xi_alloc";
const XI_OUT_OF_BOUNDS: &'static str = "_xi_out_of_bounds";
const WORD_SIZE: usize = 8;

impl Emitter {
    pub fn emit_program(self, ast: &ast::Program) -> ir::Unit<hir::Fun> {
        unimplemented!()
    }

    fn emit_fun(&mut self, fun: &ast::Fun) -> hir::Fun {
        unimplemented!()
    }

    fn emit_exp(&mut self, exp: &ast::Exp) -> hir::Exp {
        use ast::Exp::*;
        match exp {
        | Bool(false, _) => hir::Exp::Int(0),
        | Bool(true, _) => hir::Exp::Int(1),
        | Int(i, _) => hir::Exp::Int(*i),
        | Chr(c, _) => hir::Exp::Int(*c as i64),
        | Str(s, _) => {
            let symbol = symbol::intern(s);
            let label = *self.data
                .entry(symbol)
                .or_insert_with(|| operand::Label::new("STR"));
            hir::Exp::Name(label)
        }
        | Var(v, _) => hir::Exp::Temp(self.vars[v]),
        | Arr(vs, _) => {

            let alloc = Self::emit_alloc(vs.len());
            let base = hir::Exp::Temp(operand::Temp::new("ARR"));

            let mut seq = Vec::with_capacity(vs.len() + 2);
            seq.push(hir::Stm::Move(base.clone(), alloc));
            seq.push(hir::Stm::Move(base.clone(), hir::Exp::Int(vs.len() as i64)));

            for (i, v) in vs.iter().enumerate() {
                let entry = self.emit_exp(v);
                let offset = hir::Exp::Int(((i + 1) * WORD_SIZE) as i64);
                let address = hir::Exp::Mem(Box::new(
                    hir::Exp::Bin(
                        ir::Bin::Add,
                        Box::new(base.clone()),
                        Box::new(offset)
                    ),
                ));
                seq.push(hir::Stm::Move(address, entry));
            }

            hir::Exp::ESeq(
                Box::new(hir::Stm::Seq(seq)),
                Box::new(base)
            )
        }
        | _ => unimplemented!(),
        }
    }

    fn emit_alloc(length: usize) -> hir::Exp {
        let label = operand::Label::Fix(symbol::intern(XI_ALLOC));
        let alloc = hir::Exp::Name(label);
        let size = hir::Exp::Int(((length + 1) * WORD_SIZE) as i64);
        hir::Exp::Call(Box::new(alloc), vec![size])
    }

    fn emit_stm(&mut self, stm: &ast::Stm) -> hir::Stm {
        unimplemented!()
    }
}
