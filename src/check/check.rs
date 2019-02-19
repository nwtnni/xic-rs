use crate::check::Env;
use crate::data::ast;
use crate::data::typ;

pub struct Checker {
    env: Env,
}

impl Checker {

    pub fn check_call(&mut self, call: ast::Call) -> typ::Typ {
        unimplemented!()
    }

    pub fn check_dec(&mut self, dec: ast::Dec) {
        unimplemented!()
    }

    pub fn check_exp(&mut self, exp: ast::Exp) -> typ::Typ {
        unimplemented!()
    }

    pub fn check_stm(&mut self, stm: ast::Stm) -> typ::Stm {
        unimplemented!()
    }

}
