use crate::check::Env;
use crate::data::ast;
use crate::data::typ;
use crate::error;
use crate::lex;
use crate::parse;
use crate::util::symbol;

pub struct Checker {
    env: Env,
}

impl Checker {
    pub fn check_program(
        &mut self,
        lib: &std::path::Path,
        program: &ast::Program
    ) -> Result<(), error::Error> {
        for path in &program.uses {
            let path = lib.join(symbol::resolve(path.name));
            let source = std::fs::read_to_string(path)?;
            let lexer = lex::Lexer::new(&source);
            let interface = parse::InterfaceParser::new().parse(lexer)?;
            self.load_interface(&interface);
        }
        for fun in &program.funs { self.load_fun(fun)?; }
        for fun in &program.funs { self.check_fun(fun)?; }
        Ok(())
    }

    pub fn load_interface(&mut self, interface: &ast::Interface) {
        unimplemented!()
    }

    pub fn load_sig(&mut self, sig: &ast::Sig) -> Result<(), error::Error> {
        unimplemented!()
    }

    pub fn load_fun(&mut self, fun: &ast::Fun) -> Result<(), error::Error> {
        unimplemented!()
    }

    pub fn check_fun(&mut self, fun: &ast::Fun) -> Result<(), error::Error> {
        unimplemented!()
    }

    pub fn check_call(&mut self, call: &ast::Call) -> Result<typ::Typ, error::Error> {
        unimplemented!()
    }

    pub fn check_dec(&mut self, dec: &ast::Dec) -> Result<(), error::Error> {
        unimplemented!()
    }

    pub fn check_exp(&mut self, exp: &ast::Exp) -> Result<typ::Typ, error::Error> {
        unimplemented!()
    }

    pub fn check_stm(&mut self, stm: &ast::Stm) -> Result<typ::Stm, error::Error> {
        unimplemented!()
    }

}
