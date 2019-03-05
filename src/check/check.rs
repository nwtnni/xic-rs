use crate::check::Env;
use crate::data::ast;
use crate::data::ast::Callable;
use crate::data::typ;
use crate::error;
use crate::check::{Error, ErrorKind};
use crate::check::env;
use crate::lex;
use crate::parse;
use crate::util::span;
use crate::util::symbol;

macro_rules! bail {
    ($span:expr, $kind:expr) => {
        return Err(Error::new($span, $kind).into())
    }
}

macro_rules! expected {
    ($span:expr, $expected:expr, $found:expr) => {{
        let kind = ErrorKind::Mismatch {
            expected: $expected,
            found: $found,
        };
        bail!($span, kind)
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

    fn load_interface(&mut self, interface: &ast::Interface) -> Result<(), error::Error> {
        for sig in &interface.sigs {
            let (name, args, rets) = self.check_sig(sig)?;
            match self.env.get(name) {
            | Some(env::Entry::Sig(i, o)) => {
                if args.len() != i.len() {
                    bail!(sig.span, ErrorKind::NameClash)
                }
                for (arg, param) in args.iter().zip(i.iter()) {
                    if arg != param {
                        bail!(sig.span, ErrorKind::NameClash)
                    }
                }
                if rets != *o {
                    bail!(sig.span, ErrorKind::NameClash)
                }
            }
            | Some(_) => bail!(sig.span, ErrorKind::NameClash),
            | None => self.env.insert(name, env::Entry::Sig(args, rets)),
            }
        }
        Ok(())
    }

    fn load_fun(&mut self, fun: &ast::Fun) -> Result<(), error::Error> {
        let (name, args, rets) = self.check_sig(fun)?;
        match self.env.remove(name) {
        | Some(env::Entry::Sig(i, o)) => {
            if args.len() != i.len() {
                bail!(fun.span, ErrorKind::SigMismatch)
            }
            for (arg, param) in args.iter().zip(i.iter()) {
                if !arg.subtypes(param) {
                    bail!(fun.span, ErrorKind::SigMismatch)
                }
            }
            if !rets.subtypes(&o) {
                bail!(fun.span, ErrorKind::SigMismatch)
            }
            self.env.insert(name, env::Entry::Fun(i, o));
            Ok(())
        }
        | None => {
            self.env.insert(name, env::Entry::Fun(args, rets));
            Ok(())
        },
        | Some(_) => bail!(fun.span, ErrorKind::NameClash),
        }
    }

    fn check_sig<C: ast::Callable>(&self, sig: &C) -> Result<(symbol::Symbol, Vec<typ::Exp>, typ::Typ), error::Error> {
        let args = sig.args().iter()
            .map(|dec| &dec.typ)
            .map(|typ| self.check_typ(typ))
            .collect::<Result<Vec<_>, _>>()?;

        let mut rets = sig.rets().iter()
            .map(|typ| self.check_typ(&typ))
            .collect::<Result<Vec<_>, _>>()?;

        let rets = match rets.len() {
        | 0 => typ::Typ::Unit,
        | 1 => typ::Typ::Exp(rets.pop().unwrap()),
        | _ => typ::Typ::Tup(rets),
        };

        Ok((sig.name(), args, rets))
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

    fn check_fun(&mut self, fun: &ast::Fun) -> Result<(), error::Error> {
        let (params, rets) = match self.env.get(fun.name) {
        | Some(env::Entry::Fun(i, o)) => (i.clone(), o.clone()),
        | _ => panic!("[INTERNAL ERROR]: function should be bound in first pass"),
        };
        self.env.push();
        self.env.set_return(rets);
        for (arg, typ) in fun.args.iter().zip(params.iter()) {
            self.env.insert(arg.name, env::Entry::Var(typ.clone())); 
        }
        self.check_stm(&fun.body)?;
        self.env.pop();
        Ok(())
    }

    fn check_call(&self, call: &ast::Call) -> Result<typ::Typ, error::Error> {
        let (params, rets) = match self.env.get(call.name) {
        | Some(env::Entry::Sig(i, o))
        | Some(env::Entry::Fun(i, o)) => (i, o),
        | Some(_) => bail!(call.span, ErrorKind::NotFun),
        | None => bail!(call.span, ErrorKind::UnboundFun),
        };

        if call.args.len() != params.len() {
            bail!(call.span, ErrorKind::CallLength)
        }

        for (arg, param) in call.args.iter().zip(params.iter()) {
            match self.check_exp(arg)? {
            | typ::Typ::Exp(ref typ) if !typ.subtypes(param) => {
                expected!(arg.span(), typ::Typ::Exp(param.clone()), typ::Typ::Exp(typ.clone()))
            }
            | typ::Typ::Exp(_) => (),
            | _ => bail!(arg.span(), ErrorKind::NotExp),
            }
        }

        Ok(rets.clone())
    }

    fn check_dec(&mut self, dec: &ast::Dec) -> Result<typ::Typ, error::Error> {
        let typ = self.check_typ(&dec.typ)?;
        self.env.insert(dec.name, env::Entry::Var(typ.clone()));
        Ok(typ::Typ::Exp(typ))
    }

    fn check_exp(&self, exp: &ast::Exp) -> Result<typ::Typ, error::Error> {
        use ast::{Exp, Uno};
        match exp {
        | Exp::Bool(_, _) => Ok(typ::Typ::boolean()),
        | Exp::Chr(_, _) => Ok(typ::Typ::int()),
        | Exp::Str(_, _) => Ok(typ::Typ::array(typ::Exp::Int)),
        | Exp::Int(_, _) => Ok(typ::Typ::int()),
        | Exp::Var(v, span) => {
            match self.env.get(*v) {
            | Some(env::Entry::Var(typ)) => Ok(typ::Typ::Exp(typ.clone())),
            | Some(_) => bail!(*span, ErrorKind::NotVar),
            | None => bail!(*span, ErrorKind::UnboundVar),
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
                    bail!(*span, ErrorKind::IndexEmpty)
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
        | Exp::Call(call) => self.check_call(call),
        }
    }

    fn check_bin(&self, lhs: &ast::Exp, rhs: &ast::Exp, i: typ::Typ, o: typ::Typ) -> Result<typ::Typ, error::Error> {
        match (self.check_exp(lhs)?, self.check_exp(rhs)?) {
        | (ref l, ref r) if l.subtypes(&i) && r.subtypes(&i) => Ok(o),
        | (ref l, ref r) if l.subtypes(&i) => expected!(rhs.span(), i, r.clone()),
        | (l, _) => expected!(lhs.span(), i, l),
        }
    }

    fn check_stm(&mut self, stm: &ast::Stm) -> Result<typ::Stm, error::Error> {
        use ast::Stm;
        match stm {
        | Stm::Ass(lhs, rhs, _) => {
            let l = self.check_exp(lhs)?;
            let r = self.check_exp(rhs)?;
            if r.subtypes(&l) {
                Ok(typ::Stm::Unit)
            } else {
                expected!(rhs.span(), l, r)
            }
        }
        | Stm::Call(call) => {
            match self.check_call(call)? {
            | typ::Typ::Unit => Ok(typ::Stm::Unit),
            | typ => expected!(call.span, typ::Typ::Unit, typ)
            }
        }
        | Stm::Dec(dec, _) => {
            self.check_dec(dec)?;
            Ok(typ::Stm::Unit)
        }
        | Stm::Ret(rets, span) => {
            let ret = match rets.len() {
            | 0 => typ::Typ::Unit,
            | 1 => self.check_exp(&rets[0])?,
            | _ => {
                let mut typs = Vec::new();
                for ret in rets {
                    match self.check_exp(ret)? {
                    | typ::Typ::Exp(typ) => typs.push(typ),
                    | _ => bail!(ret.span(), ErrorKind::NotExp),
                    }
                }
                typ::Typ::Tup(typs)
            }
            };

            if ret.subtypes(self.env.get_return()) {
                Ok(typ::Stm::Void)
            } else {
                bail!(*span, ErrorKind::WrongReturn)
            }
        }
        | Stm::Seq(stms, _) => {
            self.env.push();
            let mut typ = typ::Stm::Unit;
            for stm in stms {
                if self.check_stm(stm)? == typ::Stm::Void {
                    typ = typ::Stm::Void;
                } else if typ == typ::Stm::Void {
                    bail!(stm.span(), ErrorKind::Unreachable)
                }
            }
            self.env.pop();
            Ok(typ)
        }
        | Stm::If(cond, pass, fail, _) => {
            match self.check_exp(cond)? {
            | typ::Typ::Exp(typ::Exp::Bool) => (),
            | typ => expected!(cond.span(), typ::Typ::boolean(), typ),
            };

            self.env.push();
            let pass = self.check_stm(pass)?;
            self.env.pop();

            if let None = fail { return Ok(pass) }

            self.env.push();
            let fail = self.check_stm(fail.as_ref().unwrap())?;
            self.env.pop();

            match (pass, fail) {
            | (typ::Stm::Void, typ::Stm::Void) => Ok(typ::Stm::Void),
            | _ => Ok(typ::Stm::Unit),
            }
        }
        | Stm::While(cond, body, _) => {
            match self.check_exp(cond)? {
            | typ::Typ::Exp(typ::Exp::Bool) => (),
            | typ => expected!(cond.span(), typ::Typ::boolean(), typ),
            };
            self.env.push();
            self.check_stm(body)?;
            self.env.pop();
            Ok(typ::Stm::Unit)
        }
        | _ => unimplemented!(),
        }
    }
}
