use crate::data::ir;
use crate::data::hir;
use crate::data::lir;

pub trait Foldable {
    fn fold(self) -> Self;
}

impl Foldable for ir::Unit<hir::Fun> {
    fn fold(self) -> Self {
        ir::Unit {
            name: self.name,
            data: self.data,
            funs: self.funs.into_iter()
                .map(|(name, fun)| (name, fun.fold()))
                .collect(),
        }
    }
}

impl Foldable for hir::Fun {
    fn fold(self) -> Self {
        hir::Fun {
            name: self.name,
            body: self.body,
        }
    }
}

impl Foldable for hir::Exp {
    fn fold(self) -> Self {
        use hir::Exp::*;
        use ir::Bin::*;
        match self {
        | Int(i) => Int(i),
        | Name(l) => Name(l),
        | Temp(t) => Temp(t),
        | Mem(e) => Mem(Box::new(e.fold())),
        | ESeq(s, e) => ESeq(Box::new(s.fold()), Box::new(e.fold())),
        | Bin(b, l, r) => {
            match (b, l.fold(), r.fold()) {
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
            | (And, Int(l), Int(r)) => Int(if (l & r) & 0b1 == 0b1 { 1 } else { 0 }),
            | (Or , Int(l), Int(r)) => Int(if (l | r) & 0b1 == 0b1 { 1 } else { 0 }),
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
}

impl Foldable for hir::Stm {
    fn fold(self) -> Self {
        use hir::Stm::*;
        match self {
        | Jump(e) => Jump(e.fold()),
        | Label(l) => Label(l),
        | Call(f, es) => Call(f.fold(), es.into_iter().map(Foldable::fold).collect()),
        | Move(d, s) => Move(d.fold(), s.fold()),
        | Return(es) => Return(es.into_iter().map(Foldable::fold).collect()),
        | Seq(ss) => Seq(ss.into_iter().map(Foldable::fold).collect()),
        | CJump(e, t, f) => {
            match e.fold() {
            | hir::Exp::Int(1) => Jump(hir::Exp::Name(t)),
            | hir::Exp::Int(0) => Jump(hir::Exp::Name(f)),
            | e => CJump(e, t, f),
            }
        }
        }
    }
}

impl Foldable for ir::Unit<lir::Fun> {
    fn fold(self) -> Self {
        ir::Unit {
            name: self.name,
            data: self.data,
            funs: self.funs.into_iter()
                .map(|(name, fun)| (name, fun.fold()))
                .collect(),
        }
    }
}

impl Foldable for lir::Fun {
    fn fold(self) -> Self {
        lir::Fun {
            name: self.name,
            body: self.body.into_iter()
                .map(Foldable::fold)
                .collect(),
        }
    }
}

impl Foldable for lir::Exp {
    fn fold(self) -> Self {
        use lir::Exp::*;
        use ir::Bin::*;
        match self {
        | Int(i) => Int(i),
        | Mem(e) => Mem(Box::new(e.fold())),
        | Name(l) => Name(l),
        | Temp(t) => Temp(t),
        | Bin(b, l, r) => {
            match (b, l.fold(), r.fold()) {
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

            | (Add, Int(0), e)
            | (Add, e, Int(0))
            | (Sub, e, Int(0))
            | (Mul, e, Int(1))
            | (Mul, Int(1), e)
            | (Div, e, Int(1))
            | (Ls,  e, Int(0))
            | (Rs,  e, Int(0))
            | (ARs, e, Int(0)) => e,

            | (Mul, _, Int(0))
            | (Mul, Int(0), _)
            | (Hul, _, Int(0))
            | (Hul, Int(0), _)
            | (Mod, _, Int(1)) => Int(0),

            | (Lt, ref t, ref u)
            | (Gt, ref t, ref u)
            | (Ne, ref t, ref u)
            | (Sub, ref t, ref u) if t == u => Int(0),

            | (Le, ref t, ref u)
            | (Ge, ref t, ref u)
            | (Eq, ref t, ref u)
            | (Div, ref t, ref u) if t == u => Int(1),

            | (b, l, r) => Bin(b, Box::new(l), Box::new(r)),
            }
        }
        }
    }
}

impl Foldable for lir::Stm {
    fn fold(self) -> Self {
        use lir::Stm::*;
        match self {
        | Jump(e) => Jump(e.fold()),
        | Call(f, es) => Call(f.fold(), es.into_iter().map(Foldable::fold).collect()),
        | Move(d, s) => Move(d.fold(), s.fold()),
        | Return(es) => Return(es.into_iter().map(Foldable::fold).collect()),
        | Label(l) => Label(l),
        | CJump(e, t, f) => {
            match e.fold() {
            | lir::Exp::Int(1) => Jump(lir::Exp::Name(t)),
            | lir::Exp::Int(0) => Jump(lir::Exp::Name(f)),
            | e => CJump(e, t, f),
            }
        }
        }
    }
}
