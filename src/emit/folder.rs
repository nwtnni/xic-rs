use crate::data::ir;
use crate::data::hir;

#[derive(Copy, Clone, Debug)]
pub struct Folder;

impl Folder {
    pub fn fold_hir_unit(unit: ir::Unit<hir::Fun>) -> ir::Unit<hir::Fun> {
        ir::Unit {
            name: unit.name,
            data: unit.data,
            funs: unit.funs.into_iter()
                .map(|(name, fun)| (name, Self::fold_hir_fun(fun)))
                .collect(),
        }
    }

    pub fn fold_hir_fun(fun: hir::Fun) -> hir::Fun {
        hir::Fun {
            name: fun.name,
            body: Self::fold_hir_stm(fun.body),
        }
    }

    pub fn fold_hir_exp(exp: hir::Exp) -> hir::Exp {
        use hir::Exp::*;
        use ir::Bin::*;
        match exp {
        | Int(i) => Int(i),
        | Name(l) => Name(l),
        | Temp(t) => Temp(t),
        | Mem(e) => Mem(Box::new(Self::fold_hir_exp(*e))),
        | ESeq(s, e) => ESeq(Box::new(Self::fold_hir_stm(*s)), Box::new(Self::fold_hir_exp(*e))),
        | Bin(b, l, r) => {
            match (b, Self::fold_hir_exp(*l), Self::fold_hir_exp(*r)) {
            | (Add, Int(l), Int(r)) => Int(l + r),
            | (Sub, Int(l), Int(r)) => Int(l - r),
            | (Mul, Int(l), Int(r)) => Int((l as i128 * r as i128) as i64),
            | (Hul, Int(l), Int(r)) => Int(((l as i128 * r as i128) >> 64) as i64),
            | (Xor, Int(l), Int(r)) => Int(l ^ r),
            | (Ls , Int(l), Int(r)) => Int(l << r),
            | (Rs , Int(l), Int(r)) => Int((l as u64 >> r) as i64),
            | (ARs, Int(l), Int(r)) => Int(l >> r),
            | (Lt , Int(l), Int(r)) => Int(if l < r  { 1 } else { 0 }),
            | (Le , Int(l), Int(r)) => Int(if l <= r { 1 } else { 0 }),
            | (Ge , Int(l), Int(r)) => Int(if l >= r { 1 } else { 0 }),
            | (Gt , Int(l), Int(r)) => Int(if l > r  { 1 } else { 0 }),
            | (Ne , Int(l), Int(r)) => Int(if l != r { 1 } else { 0 }),
            | (Eq , Int(l), Int(r)) => Int(if l == r { 1 } else { 0 }),
            | (And, Int(l), Int(r)) => Int(if l & r == 1 { 1 } else { 0 }),
            | (Or , Int(l), Int(r)) => Int(if l | r == 1 { 1 } else { 0 }),
            | (Div, Int(l), Int(r)) if r != 0 => Int(l / r),
            | (Mod, Int(l), Int(r)) if r != 0 => Int(l % r),

            | (Add, Int(0), Temp(t))
            | (Add, Temp(t), Int(0))
            | (Sub, Temp(t), Int(0))
            | (Mul, Temp(t), Int(1))
            | (Mul, Int(1), Temp(t))
            | (Div, Temp(t), Int(1))
            | (Ls,  Temp(t), Int(0))
            | (Rs,  Temp(t), Int(0))
            | (ARs, Temp(t), Int(0)) => Temp(t),

            | (Add, Int(0), Name(l))
            | (Add, Name(l), Int(0))
            | (Sub, Name(l), Int(0)) => Name(l),

            | (Mul, Temp(_), Int(0))
            | (Mul, Int(0), Temp(_))
            | (Hul, Temp(_), Int(0))
            | (Hul, Int(0), Temp(_))
            | (Mod, Temp(_), Int(1)) => Int(0),

            | (Lt, Temp(t), Temp(u))
            | (Gt, Temp(t), Temp(u))
            | (Ne, Temp(t), Temp(u))
            | (Sub, Temp(t), Temp(u)) if t == u => Int(0),

            | (Le, Temp(t), Temp(u))
            | (Ge, Temp(t), Temp(u))
            | (Eq, Temp(t), Temp(u))
            | (Div, Temp(t), Temp(u)) if t == u => Int(1),

            | (b, l, r) => Bin(b, Box::new(l), Box::new(r)),
            }
        }
        }
    }

    pub fn fold_hir_stm(stm: hir::Stm) -> hir::Stm {
        use hir::Stm::*;
        match stm {
        | Exp(e) => Exp(Self::fold_hir_exp(e)),
        | Jump(e) => Jump(Self::fold_hir_exp(e)),
        | CJump(e, t, f) => CJump(Self::fold_hir_exp(e), t, f),
        | Label(l) => Label(l),
        | Call(f, es) => Call(Self::fold_hir_exp(f), es.into_iter().map(Self::fold_hir_exp).collect()),
        | Move(d, s) => Move(Self::fold_hir_exp(d), Self::fold_hir_exp(s)),
        | Return(es) => Return(es.into_iter().map(Self::fold_hir_exp).collect()),
        | Seq(ss) => Seq(ss.into_iter().map(Self::fold_hir_stm).collect()),
        }
    }
}
