use crate::check::Env;
use crate::data::ast;
use crate::data::typ;
use crate::error;
use crate::check::{Error, ErrorKind};
use crate::check::env;
use crate::lex;
use crate::parse;
use crate::util::symbol;

macro_rules! expected {
    ($span:expr, $expected:expr, $found:expr) => {{
        let kind = ErrorKind::Mismatch {
            expected: $expected,
            found: $found,
        };
        return Err(Error::new($span, kind).into())
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

    fn check_typ(&self, typ: &ast::Typ) -> Result<typ::Exp, error::Error> {
        match typ {
        | ast::Typ::Bool(_) => Ok(typ::Exp::Bool),
        | ast::Typ::Int(_) => Ok(typ::Exp::Int),
        | ast::Typ::Arr(typ, None, _) => {
            Ok(typ::Exp::Arr(Box::new(self.check_typ(typ)?)))
        }
        | ast::Typ::Arr(typ, Some(len), _) => {
            let typ = self.check_typ(typ)?;
            match self.check_exp(len)? {
            | typ::Typ::Exp(typ::Exp::Int) => Ok(typ::Exp::Arr(Box::new(typ))),
            | typ => expected!(len.span(), typ::Typ::int(), typ),
            }
        }
        }
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

    pub fn check_exp(&self, exp: &ast::Exp) -> Result<typ::Typ, error::Error> {
        use ast::{Call, Exp, Uno};
        match exp {
        | Exp::Bool(_, _) => Ok(typ::Typ::boolean()),
        | Exp::Chr(_, _) => Ok(typ::Typ::int()),
        | Exp::Str(_, _) => Ok(typ::Typ::array(typ::Exp::Int)),
        | Exp::Int(_, _) => Ok(typ::Typ::int()),
        | Exp::Var(v, span) => {
            match self.env.get(*v) {
            | Some(env::Entry::Var(typ)) => Ok(typ::Typ::Exp(typ.clone())),
            | Some(_) => Err(Error::new(*span, ErrorKind::NotVar))?,
            | None => Err(Error::new(*span, ErrorKind::UnboundVar))?,
            }
        }
        | Exp::Arr(exps, _) => {
            let mut all = typ::Typ::any();
            for exp in exps {
                let typ = self.check_exp(exp)?;
                if all.subtypes(&typ) {
                    all = typ;
                } else {
                    expected!(exp.span(), all, typ)
                }
            }
            match all {
            | typ::Typ::Exp(typ) => Ok(typ::Typ::array(typ)),
            | _ => unreachable!(),
            }
        }
        | Exp::Bin(bin, l, r, _) if bin.is_numeric() => {
            self.check_bin(l, r, typ::Typ::int(), typ::Typ::int())
        }
        | Exp::Bin(bin, l, r, _) if bin.is_compare() => {
            self.check_bin(l, r, typ::Typ::int(), typ::Typ::boolean())
        }
        | Exp::Bin(_, l, r, _) => {
            self.check_bin(l, r, typ::Typ::boolean(), typ::Typ::boolean())
        }
        | Exp::Uno(Uno::Neg, exp, _) => {
            match self.check_exp(exp)? {
            | typ::Typ::Exp(typ::Exp::Int) => Ok(typ::Typ::int()),
            | typ => expected!(exp.span(), typ::Typ::int(), typ),
            }
        }
        | Exp::Uno(Uno::Not, exp, _) => {
            match self.check_exp(exp)? {
            | typ::Typ::Exp(typ::Exp::Bool) => Ok(typ::Typ::boolean()),
            | typ => expected!(exp.span(), typ::Typ::boolean(), typ),
            }
        }
        | Exp::Idx(arr, idx, span) => {
            match (self.check_exp(arr)?, self.check_exp(idx)?) {
            | (typ::Typ::Exp(typ::Exp::Arr(typ)), typ::Typ::Exp(typ::Exp::Int)) => {
                if *typ == typ::Exp::Any {
                    let kind = ErrorKind::IndexEmpty;
                    Err(Error::new(*span, kind).into())
                } else {
                    Ok(typ::Typ::Exp(*typ))
                }
            }
            | (typ::Typ::Exp(typ::Exp::Arr(_)), typ) => {
                expected!(idx.span(), typ::Typ::int(), typ)
            }
            | (typ, _) => {
                expected!(arr.span(), typ::Typ::any(), typ)
            }
            }
        }
        | Exp::Call(Call { name, args, span }) => {
            let (i, o) = match self.env.get(*name) {
            | Some(env::Entry::Fun(i, o)) => (i, o),
            | Some(_) => return Err(Error::new(*span, ErrorKind::NotFun).into()),
            | None => return Err(Error::new(*span, ErrorKind::UnboundFun).into()),
            };
            
            // Type check each argument to an expression type
            let mut typs = Vec::new();
            for arg in args {
                match self.check_exp(arg)? {
                | typ::Typ::Exp(typ) => typs.push((typ, arg.span())),
                | _ => return Err(Error::new(arg.span(), ErrorKind::NotExp).into()),
                }
            }
            
            match i {
            | typ::Typ::Exp(i) => {
                if typs.len() != 1 {
                    Err(Error::new(*span, ErrorKind::CallLength).into())
                } else if !typs[0].0.subtypes(i) {
                    expected!(typs[0].1, typ::Typ::Exp(i.clone()), typ::Typ::Exp(typs[0].0))
                } else {
                    Ok(o.clone())
                }
            }
            | typ::Typ::Tup(is) => {
                if typs.len() != is.len() {
                    return Err(Error::new(*span, ErrorKind::CallLength).into())
                }
                for ((typ, span), i) in typs.into_iter().zip(is.iter()) {
                    if !typ.subtypes(i) {
                        expected!(span, typ::Typ::Exp(i.clone()), typ::Typ::Exp(typ))
                    }
                }
                Ok(o.clone())
            }
            | _ => unreachable!(),
            }
        }
        }
    }

    fn check_bin(&self, lhs: &ast::Exp, rhs: &ast::Exp, i: typ::Typ, o: typ::Typ) -> Result<typ::Typ, error::Error> {
        match (self.check_exp(lhs)?, self.check_exp(rhs)?) {
        | (ref l, ref r) if l.subtypes(&i) && r.subtypes(&i) => Ok(o),
        | (ref l, ref r) if l.subtypes(&i) => expected!(rhs.span(), i, r.clone()),
        | (l, _) => expected!(lhs.span(), i, l),
        }
    }

    pub fn check_stm(&mut self, stm: &ast::Stm) -> Result<typ::Stm, error::Error> {
        unimplemented!()
    }

}
