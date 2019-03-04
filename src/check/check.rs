use crate::check::Env;
use crate::data::ast;
use crate::data::typ;
use crate::error;
use crate::check;
use crate::check::env;
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

    pub fn check_exp(&mut self, exp: &ast::Exp) -> Result<typ::Exp, error::Error> {
        match exp {
        | ast::Exp::Bool(_, _) => Ok(typ::Exp::Bool),
        | ast::Exp::Chr(_, _) => Ok(typ::Exp::Int),
        | ast::Exp::Str(_, _) => Ok(typ::Exp::Arr(Box::new(typ::Exp::Int))),
        | ast::Exp::Int(_, _) => Ok(typ::Exp::Int),
        | ast::Exp::Var(v, span) => {
            match self.env.get(*v) {
            | Some(env::Entry::Var(typ)) => Ok(typ.clone()),
            | Some(_) => Err(check::Error::new(*span, check::ErrorKind::NotVarTyp))?,
            | None => Err(check::Error::new(*span, check::ErrorKind::UnboundVar))?,
            }
        }
        | _ => unimplemented!(),
        }
    }

    pub fn check_stm(&mut self, stm: &ast::Stm) -> Result<typ::Stm, error::Error> {
        unimplemented!()
    }

}
