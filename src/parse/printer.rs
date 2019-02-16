use crate::sexp::{Sexp, Serialize};
use crate::ast;
use crate::util::*;

impl Serialize for ast::Interface {
    fn sexp(&self) -> Sexp {
        self.sigs.sexp()
    }
}

impl Serialize for ast::Program {
    fn sexp(&self) -> Sexp {
        vec![
            self.uses.sexp(),
            self.funs.sexp(),
        ].sexp_move()
    }
}

impl Serialize for ast::Use {
    fn sexp(&self) -> Sexp {
        vec!["use".sexp(), self.name.sexp()].sexp_move()
    }
}

impl Serialize for ast::Sig {
    fn sexp(&self) -> Sexp {
        vec![
            self.name.sexp(),
            self.args.sexp(),
            self.rets.sexp(),
        ].sexp_move()
    }
}

impl Serialize for ast::Fun {
    fn sexp(&self) -> Sexp {
        vec![
            self.name.sexp(),
            self.args.sexp(),
            self.rets.sexp(),
            self.body.sexp(),
        ].sexp_move()
    }
}

impl Serialize for ast::Typ {
    fn sexp(&self) -> Sexp {
        use ast::Typ::*;
        match self {
        | Bool(_) => "bool".sexp(),
        | Int(_) => "int".sexp(),
        | Arr(typ, None, _) => vec!["[]".sexp(), typ.sexp()].sexp_move(),
        | Arr(typ, Some(exp), _) => vec!["[]".sexp(), typ.sexp(), exp.sexp()].sexp_move(),
        }
    }
}

impl Serialize for ast::Bin {
    fn sexp(&self) -> Sexp {
        use ast::Bin::*;
        match self {
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
        }.sexp()
    }
}

impl Serialize for ast::Uno {
    fn sexp(&self) -> Sexp {
        use ast::Uno::*;
        match self {
        | Neg => "-",
        | Not => "!",
        }.sexp()
    }
}

impl Serialize for ast::Exp {
    fn sexp(&self) -> Sexp {
        use ast::Exp::*;
        match self {
        | Bool(false, _) => "false".sexp(),
        | Bool(true, _) => "true".sexp(),
        | Chr(c, _) => c.to_string().sexp(),
        | Str(s, _) => s.to_string().sexp(),
        | Int(i, _) => i.to_string().sexp(),
        | Var(v, _) => v.sexp(),
        | Arr(exps, _) => exps.sexp(),
        | Bin(bin, lhs, rhs, _) => vec![bin.sexp(), lhs.sexp(), rhs.sexp()].sexp_move(),
        | Uno(uno, exp, _) => vec![uno.sexp(), exp.sexp()].sexp_move(),
        | Idx(arr, idx, _) => vec!["[]".sexp(), arr.sexp(), idx.sexp()].sexp_move(),
        | Call(call) => call.sexp(),
        }
    }
}

impl Serialize for ast::Dec {
    fn sexp(&self) -> Sexp {
        vec![self.name.sexp(), self.typ.sexp()].sexp_move()
    }
}

impl Serialize for ast::Call {
    fn sexp(&self) -> Sexp {
        let mut args = self.args.iter()
            .map(Serialize::sexp)
            .collect::<Vec<_>>();
        args.insert(0, self.name.sexp());
        args.sexp_move()
    }
}

impl Serialize for ast::Stm {
    fn sexp(&self) -> Sexp {
        use ast::Stm::*;
        match self {
        | Ass(lhs, rhs, _) => vec!["=".sexp(), lhs.sexp(), rhs.sexp()].sexp_move(),
        | Call(call) => call.sexp(),
        | Init(decs, call, _) => {
            let decs = decs.iter()
                .map(|dec| dec.as_ref().map(Serialize::sexp).unwrap_or_else(|| "_".sexp()))
                .collect::<Vec<_>>();
            vec!["=".sexp(), decs.sexp(), call.sexp()].sexp_move()
        }
        | Dec(dec, _) => dec.sexp(),
        | Ret(exps, _) => {
            std::iter::once("return".sexp())
                .chain(exps.iter().map(Serialize::sexp))
                .collect::<Vec<_>>()
                .tap(Sexp::List)
        }
        | Seq(stms, _) => stms.sexp(),
        | If(cond, pass, Some(fail), _) => {
            vec!["if".sexp(), cond.sexp(), pass.sexp(), fail.sexp()].sexp_move()
        }
        | If(cond, pass, None, _) => {
            vec!["if".sexp(), cond.sexp(), pass.sexp()].sexp_move()
        }
        | While(cond, body, _) => {
            vec!["while".sexp(), cond.sexp(), body.sexp()].sexp_move()
        }
        }
    }
}
