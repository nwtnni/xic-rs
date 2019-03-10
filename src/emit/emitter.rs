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
}

const XI_ALLOC: &'static str = "_xi_alloc";
const XI_OUT_OF_BOUNDS: &'static str = "_xi_out_of_bounds";

impl Emitter {
    pub fn emit_program(&self, ast: &ast::Program) -> ir::Unit<hir::Fun> {
        unimplemented!()    
    }

    pub fn emit_fun(&self, fun: &ast::Fun) -> hir::Fun {
        unimplemented!()    
    }

    pub fn emit_exp(&self, exp: &ast::Exp) -> hir::Exp {
        use ast::Exp::*;
        match exp {
        | Bool(false, _) => hir::Exp::Int(0),
        | Bool(true, _) => hir::Exp::Int(1),
        | Chr(c, _) => hir::Exp::Int(*c as i64),
        | _ => unimplemented!(),
        }
    }

    pub fn emit_stm(&self, stm: &ast::Stm) -> hir::Stm {
        unimplemented!()    
    }
}
