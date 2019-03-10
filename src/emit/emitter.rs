use crate::check;
use crate::data::ast;
use crate::data::ir;
use crate::data::hir;

#[derive(Debug)]
pub struct Emitter {
    env: check::Env,
}

impl Emitter {
    pub fn emit_program(&self, ast: &ast::Program) -> ir::Unit<hir::Fun> {
        unimplemented!()    
    }

    pub fn emit_fun(&self, fun: &ast::Fun) -> hir::Fun {
        unimplemented!()    
    }

    pub fn emit_exp(&self, exp: &ast::Exp) -> hir::Exp {
        unimplemented!()    
    }

    pub fn emit_stm(&self, stm: &ast::Stm) -> hir::Stm {
        unimplemented!()    
    }
}
