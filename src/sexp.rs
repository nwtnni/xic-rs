use std::borrow::Cow;

use crate::ast;
use crate::symbol;
use crate::util::{Conv, Tap};

#[derive(Clone, Debug)]
pub enum Sexp {
    Atom(Cow<'static, str>),
    List(Vec<Sexp>), 
}

impl From<&symbol::Symbol> for Sexp {
    fn from(s: &symbol::Symbol) -> Sexp { Sexp::Atom(Cow::from(symbol::resolve(*s))) }
}

impl From<&'static str> for Sexp {
    fn from(s: &'static str) -> Sexp { Sexp::Atom(Cow::from(s)) }
}

impl From<&String> for Sexp {
    fn from(s: &String) -> Sexp { Sexp::Atom(Cow::from(s.clone())) }
}

impl From<String> for Sexp {
    fn from(s: String) -> Sexp { Sexp::Atom(Cow::from(s)) }
}

impl From<Vec<Sexp>> for Sexp {
    fn from(v: Vec<Sexp>) -> Sexp { Sexp::List(v) }
}

impl From<&Box<ast::Exp>> for Sexp {
    fn from(e: &Box<ast::Exp>) -> Sexp { Sexp::from(&**e) }
}

impl From<&ast::Interface> for Sexp {
    fn from(interface: &ast::Interface) -> Sexp {
        interface.sigs.iter()
            .map(Conv::conv)
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<&ast::Program> for Sexp {
    fn from(program: &ast::Program) -> Sexp {
        vec![
            program.uses.iter().map(Conv::conv).collect::<Vec<_>>().into(),
            program.funs.iter().map(Conv::conv).collect::<Vec<_>>().into(),
        ].into()
    }
}

impl From<&ast::Use> for Sexp {
    fn from(uses: &ast::Use) -> Sexp {
        vec!["use".into(), (&uses.name).into()].into()
    }
}

impl From<&ast::Sig> for Sexp {
    fn from(sig: &ast::Sig) -> Sexp {
        vec![
            (&sig.name).into(),
            sig.args.iter().map(Conv::conv).collect::<Vec<_>>().into(),
            sig.rets.iter().map(Conv::conv).collect::<Vec<_>>().into(),
        ].into()
    }
}

impl From<&ast::Fun> for Sexp {
    fn from(fun: &ast::Fun) -> Sexp {
        vec![
            (&fun.name).into(),
            fun.args.iter().map(Conv::conv).collect::<Vec<_>>().into(),
            fun.rets.iter().map(Conv::conv).collect::<Vec<_>>().into(),
            (&fun.body).into(),
        ].into()
    }
}

impl From<&ast::Typ> for Sexp {
    fn from(typ: &ast::Typ) -> Sexp {
        use ast::Typ::*;
        match typ {
        | Bool(_) => "bool".into(),
        | Int(_) => "int".into(),
        | Arr(typ, None, _) => vec!["[]".into(), (&**typ).into()].into(),
        | Arr(typ, Some(exp), _) => vec!["[]".into(), (&**typ).into(), exp.into()].into(),
        }
    }
}

impl From<&ast::Bin> for Sexp {
    fn from(bin: &ast::Bin) -> Sexp {
        use ast::Bin::*;
        match bin {
        | Mul => "*",
        | Hul => "*>>",
        | Div => "/",
        | Mod => "%",
        | Add => "+",
        | Sub => "-",
        | Lt => "<",
        | Le => "<=",
        | Ge => ">=",
        | Gt => ">",
        | Eq => "==",
        | Ne => "!=",
        | And => "&",
        | Or => "|",
        }.into()
    }
}

impl From<&ast::Uno> for Sexp {
    fn from(uno: &ast::Uno) -> Sexp {
        use ast::Uno::*;
        match uno {
        | Neg => "-",
        | Not => "!",
        }.into()
    }
}

impl From<&ast::Exp> for Sexp {
    fn from(exp: &ast::Exp) -> Sexp {
        use ast::Exp::*;
        match exp {
        | Bool(false, _) => "false".into(),
        | Bool(true, _) => "true".into(),
        | Chr(c, _) => c.to_string().into(),
        | Str(s, _) => s.to_string().into(),
        | Int(i, _) => i.to_string().into(),
        | Var(v, _) => v.into(),
        | Arr(exps, _) => exps.iter().map(Conv::conv).collect::<Vec<_>>().into(),
        | Bin(bin, lhs, rhs, _) => vec![bin.into(), lhs.into(), rhs.into()].into(),
        | Uno(uno, exp, _) => vec![uno.into(), exp.into()].into(),
        | Idx(arr, idx, _) => vec!["[]".into(), arr.into(), idx.into()].into(),
        | Call(call) => call.into(),
        }
    }
}

impl From<&ast::Dec> for Sexp {
    fn from(dec: &ast::Dec) -> Sexp {
        vec![(&dec.name).into(), (&dec.typ).into()].into()
    }
}

impl From<&ast::Call> for Sexp {
    fn from(call: &ast::Call) -> Sexp {
        let mut args: Vec<Sexp> = call.args.iter()
            .map(Conv::conv)
            .collect::<Vec<_>>()
            .into();
        args.insert(0, (&call.name).into());
        args.into()
    }
}

impl From<&ast::Stm> for Sexp {
    fn from(stm: &ast::Stm) -> Sexp {
        use ast::Stm::*;
        match stm {
        | Ass(lhs, rhs, _) => vec!["=".into(), lhs.into(), rhs.into()].into(),
        | Call(call) => call.into(),
        | Init(decs, call, _) => {
            let decs = decs.iter()
                .map(|dec| dec.as_ref().map(Conv::conv::<Sexp>).unwrap_or_else(|| "_".into()))
                .collect::<Vec<_>>();
            vec!["=".into(), decs.into(), call.into()].into()
        }
        | Dec(dec, _) => dec.into(),
        | Ret(exps, _) => {
            std::iter::once("return".into())
                .chain(exps.iter().map(Conv::conv))
                .collect::<Vec<_>>()
                .tap(Conv::conv)
        }
        | Seq(stms, _) => {
            stms.iter()
                .map(Conv::conv)
                .collect::<Vec<_>>()
                .tap(Conv::conv)
        }
        | If(cond, pass, Some(fail), _) => {
            vec!["if".into(), cond.into(), (&**pass).into(), (&**fail).into()].into()
        }
        | If(cond, pass, None, _) => {
            vec!["if".into(), cond.into(), (&**pass).into()].into()
        }
        | While(cond, body, _) => {
            vec!["while".into(), cond.into(), (&**body).into()].into()
        }
        }
    }
}
