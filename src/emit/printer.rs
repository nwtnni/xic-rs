use crate::data::ir;
use crate::data::lir;
use crate::data::hir;
use crate::data::operand;
use crate::util::sexp::{Sexp, Serialize};
use crate::util::symbol;

impl<T: ir::IR + Serialize> Serialize for ir::Unit<T> {
    fn sexp(&self) -> Sexp {
        std::iter::once("COMPUNIT".sexp())
            .chain(std::iter::once(self.name.sexp()))
            .chain(self.funs.values().map(|fun| fun.sexp()))
            .collect::<Vec<_>>()
            .sexp_move()
    }
}

impl Serialize for hir::Fun {
    fn sexp(&self) -> Sexp {
        vec![
            "FUNC".sexp(),
            self.name.sexp(),
            self.body.sexp(),
        ].sexp_move()
    }
}

impl Serialize for hir::Exp {
    fn sexp(&self) -> Sexp {
        use hir::Exp::*;
        match self {
        | Int(i) => i.sexp(),
        | Mem(e) => vec!["MEM".sexp(), e.sexp()].sexp_move(),
        | Bin(b, l, r) => vec![b.sexp(), l.sexp(), r.sexp()].sexp_move(),
        | Name(l) => vec!["NAME".sexp(), l.sexp()].sexp_move(),
        | Call(f, args) => {
            std::iter::once("CALL".sexp())
                .chain(std::iter::once(f.sexp()))
                .chain(args.iter().map(|arg| arg.sexp()))
                .collect::<Vec<_>>()
                .sexp_move()
        }
        | Temp(t) => vec!["TEMP".sexp(), t.sexp()].sexp_move(),
        | ESeq(s, e) => vec!["ESEQ".sexp(), s.sexp(), e.sexp()].sexp_move(),
        }
    }
}

impl Serialize for hir::Stm {
    fn sexp(&self) -> Sexp {
        use hir::Stm::*;
        match self {
        | Exp(e) => vec!["EXP".sexp(), e.sexp()].sexp_move(),
        | Jump(e) => vec!["JUMP".sexp(), e.sexp()].sexp_move(),
        | CJump(b, l, r, t, f) => {
            vec![
                "CJUMP".sexp(),
                vec![b.sexp(), l.sexp(), r.sexp()].sexp_move(),
                t.sexp(),
                f.sexp(),
            ].sexp_move()
        }
        | Label(l) => vec!["LABEL".sexp(), l.sexp()].sexp_move(),
        | Move(d, s) => vec!["MOVE".sexp(), d.sexp(), s.sexp()].sexp_move(),
        | Return(exps) => {
            std::iter::once("RETURN".sexp())
                .chain(exps.iter().map(|exp| exp.sexp()))
                .collect::<Vec<_>>()
                .sexp_move()
        }
        | Seq(stms) => {
            std::iter::once("SEQ".sexp())
                .chain(stms.iter().map(|stm| stm.sexp()))
                .collect::<Vec<_>>()
                .sexp_move()
        }
        }
    }
}

impl Serialize for lir::Fun {
    fn sexp(&self) -> Sexp {
        vec![
            "FUNC".sexp(),
            self.name.sexp(),
            std::iter::once("SEQ".sexp())
                .chain(self.body.iter().map(|stm| stm.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
        ].sexp_move()
    }
}

impl Serialize for lir::Exp {
    fn sexp(&self) -> Sexp {
        use lir::Exp::*;
        match self {
        | Int(i) => i.sexp(),
        | Mem(e) => vec!["MEM".sexp(), e.sexp()].sexp_move(),
        | Bin(b, l, r) => vec![b.sexp(), l.sexp(), r.sexp()].sexp_move(),
        | Name(l) => vec!["NAME".sexp(), l.sexp()].sexp_move(),
        | Temp(t) => vec!["TEMP".sexp(), t.sexp()].sexp_move(),
        }
    }
}

impl Serialize for lir::Stm {
    fn sexp(&self) -> Sexp {
        use lir::Stm::*;
        match self {
        | Call(f, args) => {
            std::iter::once("CALL".sexp())
                .chain(std::iter::once(f.sexp()))
                .chain(args.iter().map(|arg| arg.sexp()))
                .collect::<Vec<_>>()
                .sexp_move()
        }
        | Jump(e) => vec!["JUMP".sexp(), e.sexp()].sexp_move(),
        | CJump(b, l, r, t) => {
            vec![
                "CJUMP".sexp(),
                vec![b.sexp(), l.sexp(), r.sexp()].sexp_move(),
                t.sexp(),
            ].sexp_move()
        }
        | Label(l) => vec!["LABEL".sexp(), l.sexp()].sexp_move(),
        | Move(d, s) => vec!["MOVE".sexp(), d.sexp(), s.sexp()].sexp_move(),
        | Return(exps) => {
            std::iter::once("RETURN".sexp())
                .chain(exps.iter().map(|exp| exp.sexp()))
                .collect::<Vec<_>>()
                .sexp_move()
        }
        }
    }
}

impl Serialize for ir::Bin {
    fn sexp(&self) -> Sexp {
        use ir::Bin::*;
        match self {
        | Add => "ADD".sexp(),
        | Sub => "SUB".sexp(),
        | Mul => "MUL".sexp(),
        | Hul => "HMUL".sexp(),
        | Div => "DIV".sexp(),
        | Mod => "MOD".sexp(),
        | And => "AND".sexp(),
        | Or  => "OR".sexp(),
        | Xor => "XOR".sexp(),
        | Ls  => "LSHIFT".sexp(),
        | Rs  => "RSHIFT".sexp(),
        | ARs => "ARSHIFT".sexp(),
        }
    }
}

impl Serialize for ir::Rel {
    fn sexp(&self) -> Sexp {
        use ir::Rel::*;
        match self {
        | Lt => "LT".sexp(),
        | Le => "LEQ".sexp(),
        | Ge => "GEQ".sexp(),
        | Gt => "GT".sexp(),
        | Ne => "NEQ".sexp(),
        | Eq => "EQ".sexp(),
        }
    }
}

impl Serialize for operand::Label {
    fn sexp(&self) -> Sexp {
        use operand::Label::*;
        match self {
        | Fix(sym) => sym.sexp(),
        | Gen(sym, i) => format!("{}_{}", symbol::resolve(*sym), i).sexp_move(),
        }
    }
}

impl Serialize for operand::Temp {
    fn sexp(&self) -> Sexp {
        use operand::Temp::*;
        match self {
        | Arg(i) => format!("_ARG{}", i).sexp_move(),
        | Ret(i) => format!("_RET{}", i).sexp_move(),
        | Gen(sym, i) => format!("{}_{}", symbol::resolve(*sym), i).sexp_move(),
        | Reg(_) => panic!("[INTERNAL ERROR]: shouldn't be any registers in IR"),
        }
    }
}
