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

    fn emit_exp(&mut self, exp: &ast::Exp) -> hir::Tree {
        use ast::Exp::*;
        match exp {
        | Bool(false, _) => hir::Exp::Int(0).into(),
        | Bool(true, _) => hir::Exp::Int(1).into(),
        | Int(i, _) => hir::Exp::Int(*i).into(),
        | Chr(c, _) => hir::Exp::Int(*c as i64).into(),
        | Str(s, _) => {
            let symbol = symbol::intern(s);
            let label = *self.data
                .entry(symbol)
                .or_insert_with(|| operand::Label::new("STR"));
            hir::Exp::Name(label).into()
        }
        | Var(v, _) => hir::Exp::Temp(self.vars[v]).into(),
        | Arr(vs, _) => {

            let alloc = Self::emit_alloc(vs.len());
            let base = hir::Exp::Temp(operand::Temp::new("ARR"));

            let mut seq = Vec::with_capacity(vs.len() + 2);
            seq.push(hir::Stm::Move(base.clone(), alloc));
            seq.push(hir::Stm::Move(base.clone(), hir::Exp::Int(vs.len() as i64)));

            for (i, v) in vs.iter().enumerate() {
                let entry = self.emit_exp(v).into();
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
            ).into()
        }
        | Bin(bin, lhs, rhs, _) => {
            let lhs = self.emit_exp(lhs);
            let rhs = self.emit_exp(rhs);

            if let Some(binop) = match bin {
            | ast::Bin::Mul => Some(ir::Bin::Mul),
            | ast::Bin::Hul => Some(ir::Bin::Hul),
            | ast::Bin::Mod => Some(ir::Bin::Mod),
            | ast::Bin::Div => Some(ir::Bin::Div),
            | ast::Bin::Add => Some(ir::Bin::Add),
            | ast::Bin::Sub => Some(ir::Bin::Sub),
            | _ => None,
            } {
                return hir::Exp::Bin(
                    binop,
                    Box::new(lhs.into()),
                    Box::new(rhs.into()),
                ).into()
            }

            if let Some(relop) = match bin {
            | ast::Bin::Lt => Some(ir::Rel::Lt),
            | ast::Bin::Le => Some(ir::Rel::Le),
            | ast::Bin::Ge => Some(ir::Rel::Ge),
            | ast::Bin::Gt => Some(ir::Rel::Gt),
            | ast::Bin::Ne => Some(ir::Rel::Ne),
            | ast::Bin::Eq => Some(ir::Rel::Eq),
            | _ => None,
            } {
                return hir::Tree::Cx(Box::new(move |t, f| {
                    hir::Stm::CJump(relop, lhs.into(), rhs.into(), t, f)
                }))
            }

            match bin {
            | ast::Bin::And => {
                hir::Tree::Cx(Box::new(move |t, f| {
                    let and = operand::Label::new("AND");
                    hir::Stm::Seq(vec![
                        hir::Con::from(lhs)(and, f),
                        hir::Stm::Label(and),
                        hir::Con::from(rhs)(t, f),
                    ])
                }))
            }
            | ast::Bin::Or => {
                hir::Tree::Cx(Box::new(move |t, f| {
                    let or = operand::Label::new("OR");
                    hir::Stm::Seq(vec![
                        hir::Con::from(lhs)(t, or),
                        hir::Stm::Label(or),
                        hir::Con::from(rhs)(t, f),
                    ])
                }))
            }
            | _ => panic!("[INTERNAL ERROR]: missing binary operator in IR emission"),
            }
        }
        | Uno(ast::Uno::Neg, exp, _) => {
            hir::Exp::Bin(
                ir::Bin::Sub, 
                Box::new(hir::Exp::Int(0)),
                Box::new(self.emit_exp(exp).into()),
            ).into()
        }
        | Uno(ast::Uno::Not, exp, _) => {
            hir::Exp::Bin(
                ir::Bin::Xor,
                Box::new(hir::Exp::Int(1)),
                Box::new(self.emit_exp(exp).into()),
            ).into()
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
