use crate::check::Env;
use crate::data::ast;
use crate::data::typ;
use crate::error;
use crate::check;
use crate::check::env;
use crate::lex;
use crate::parse;
use crate::util::symbol;

macro_rules! expected {
    ($span:expr, $expected:expr, $found:expr) => {{
        let kind = check::ErrorKind::Mismatch {
            expected: typ::Typ::Exp($expected),
            found: typ::Typ::Exp($found),
        };
        return Err(check::Error::new($span, kind).into())
    }}
}

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

        use ast::{Bin, Exp};

        match exp {
        | Exp::Bool(_, _) => Ok(typ::Exp::Bool),
        | Exp::Chr(_, _) => Ok(typ::Exp::Int),
        | Exp::Str(_, _) => Ok(typ::Exp::Arr(Box::new(typ::Exp::Int))),
        | Exp::Int(_, _) => Ok(typ::Exp::Int),
        | Exp::Var(v, span) => {
            match self.env.get(*v) {
            | Some(env::Entry::Var(typ)) => Ok(typ.clone()),
            | Some(_) => Err(check::Error::new(*span, check::ErrorKind::NotVarTyp))?,
            | None => Err(check::Error::new(*span, check::ErrorKind::UnboundVar))?,
            }
        }
        | Exp::Arr(exps, _) => {
            let mut all = typ::Exp::Any;
            for exp in exps {
                let typ = self.check_exp(exp)?;
                if all.subtypes(&typ) {
                    all = typ;
                } else {
                    expected!(exp.span(), all, typ)
                }
            }
            Ok(typ::Exp::Arr(Box::new(all)))
        }
        | Exp::Bin(bin, l, r, _) if bin.is_numeric() => {
            self.check_bin(l, r, typ::Exp::Int, typ::Exp::Int)
        }
        | Exp::Bin(bin, l, r, _) if bin.is_compare() => {
            self.check_bin(l, r, typ::Exp::Int, typ::Exp::Bool)
        }
        | Exp::Bin(_, l, r, _) => {
            self.check_bin(l, r, typ::Exp::Bool, typ::Exp::Bool)
        }
        | _ => unimplemented!(),
        }
    }

    fn check_bin(&mut self, lhs: &ast::Exp, rhs: &ast::Exp, i: typ::Exp, o: typ::Exp) -> Result<typ::Exp, error::Error> {
        match (self.check_exp(lhs)?, self.check_exp(rhs)?) {
        | (ref l, ref r) if l.subtypes(&i) && r.subtypes(&i) => Ok(o),
        | (ref l, ref r) if l.subtypes(&i) => expected!(rhs.span(), r.clone(), i),
        | (l, _) => expected!(lhs.span(), l, i),
        }
    }

    pub fn check_stm(&mut self, stm: &ast::Stm) -> Result<typ::Stm, error::Error> {
        unimplemented!()
    }

}
