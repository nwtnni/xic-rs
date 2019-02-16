use std::borrow::Cow;

use crate::ast;
use crate::symbol;
use crate::util::Conv;

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
        | Call(ast::Call { name, args, .. }) => {
            let mut args: Vec<Sexp> = args.into_iter().map(Conv::conv).collect::<Vec<_>>().into();
            args.insert(0, name.into());
            args.into()
        }
        }
    }
}
