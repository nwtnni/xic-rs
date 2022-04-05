use crate::data::ast;
use crate::util;
use crate::util::sexp::{Serialize, Sexp};
use crate::util::Tap;

impl Serialize for ast::Interface {
    fn sexp(&self) -> Sexp {
        self.sigs.sexp()
    }
}

impl Serialize for ast::Program {
    fn sexp(&self) -> Sexp {
        [self.uses.sexp(), self.funs.sexp()].sexp_move()
    }
}

impl Serialize for ast::Use {
    fn sexp(&self) -> Sexp {
        ["use".sexp(), self.name.sexp()].sexp_move()
    }
}

impl Serialize for ast::Sig {
    fn sexp(&self) -> Sexp {
        [self.name.sexp(), self.args.sexp(), self.rets.sexp()].sexp_move()
    }
}

impl Serialize for ast::Fun {
    fn sexp(&self) -> Sexp {
        [
            self.name.sexp(),
            self.args.sexp(),
            self.rets.sexp(),
            self.body.sexp(),
        ]
        .sexp_move()
    }
}

impl Serialize for ast::Typ {
    fn sexp(&self) -> Sexp {
        use ast::Typ::*;
        match self {
            Bool(_) => "bool".sexp(),
            Int(_) => "int".sexp(),
            Arr(typ, None, _) => ["[]".sexp(), typ.sexp()].sexp_move(),
            Arr(typ, Some(exp), _) => ["[]".sexp(), typ.sexp(), exp.sexp()].sexp_move(),
        }
    }
}

impl Serialize for ast::Bin {
    fn sexp(&self) -> Sexp {
        use ast::Bin::*;
        match self {
            Mul => "*",
            Hul => "*>>",
            Div => "/",
            Mod => "%",
            Add => "+",
            Sub => "-",
            Lt => "<",
            Le => "<=",
            Ge => ">=",
            Gt => ">",
            Eq => "==",
            Ne => "!=",
            And => "&",
            Or => "|",
        }
        .sexp()
    }
}

impl Serialize for ast::Uno {
    fn sexp(&self) -> Sexp {
        use ast::Uno::*;
        match self {
            Neg => "-",
            Not => "!",
        }
        .sexp()
    }
}

impl Serialize for ast::Exp {
    fn sexp(&self) -> Sexp {
        use ast::Exp::*;
        match self {
            Bool(false, _) => "false".sexp(),
            Bool(true, _) => "true".sexp(),
            Chr(c, _) => match util::unescape_char(*c) {
                Some(s) => format!("\'{}\'", s).sexp_move(),
                None => format!("\'{}\'", c).sexp_move(),
            },
            Str(s, _) => format!("\"{}\"", util::unescape_str(s)).sexp_move(),
            Int(i, _) if *i < 0 => {
                ["-".sexp(), (-(*i as i128)).to_string().sexp_move()].sexp_move()
            }
            Int(i, _) => i.to_string().sexp_move(),
            Var(v, _) => v.sexp(),
            Arr(exps, _) => exps.sexp(),
            Bin(bin, lhs, rhs, _) => [bin.sexp(), lhs.sexp(), rhs.sexp()].sexp_move(),
            Uno(uno, exp, _) => [uno.sexp(), exp.sexp()].sexp_move(),
            Idx(arr, idx, _) => ["[]".sexp(), arr.sexp(), idx.sexp()].sexp_move(),
            Call(call) => call.sexp(),
        }
    }
}

impl Serialize for ast::Dec {
    fn sexp(&self) -> Sexp {
        [self.name.sexp(), self.typ.sexp()].sexp_move()
    }
}

impl Serialize for ast::Call {
    fn sexp(&self) -> Sexp {
        let mut args = self.args.iter().map(Serialize::sexp).collect::<Vec<_>>();
        args.insert(0, self.name.sexp());
        args.sexp_move()
    }
}

impl Serialize for ast::Stm {
    fn sexp(&self) -> Sexp {
        use ast::Stm::*;
        match self {
            Ass(lhs, rhs, _) => ["=".sexp(), lhs.sexp(), rhs.sexp()].sexp_move(),
            Call(call) => call.sexp(),
            Init(decs, call, _) => {
                let mut decs = decs
                    .iter()
                    .map(|dec| {
                        dec.as_ref()
                            .map(Serialize::sexp)
                            .unwrap_or_else(|| "_".sexp())
                    })
                    .collect::<Vec<_>>();
                let decs = if decs.len() == 1 {
                    decs.remove(0)
                } else {
                    Sexp::List(decs)
                };
                ["=".sexp(), decs.sexp(), call.sexp()].sexp_move()
            }
            Dec(dec, _) => dec.sexp(),
            Ret(exps, _) => std::iter::once("return".sexp())
                .chain(exps.iter().map(Serialize::sexp))
                .collect::<Vec<_>>()
                .tap(Sexp::List),
            Seq(stms, _) => stms.sexp(),
            If(cond, pass, Some(fail), _) => {
                ["if".sexp(), cond.sexp(), pass.sexp(), fail.sexp()].sexp_move()
            }
            If(cond, pass, None, _) => ["if".sexp(), cond.sexp(), pass.sexp()].sexp_move(),
            While(cond, body, _) => ["while".sexp(), cond.sexp(), body.sexp()].sexp_move(),
        }
    }
}
