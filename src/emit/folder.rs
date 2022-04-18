use crate::data::hir;
use crate::data::ir;
use crate::data::lir;

pub trait Foldable {
    fn fold(self) -> Self;
}

impl Foldable for ir::Unit<hir::Function> {
    fn fold(self) -> Self {
        ir::Unit {
            name: self.name,
            data: self.data,
            functions: self
                .functions
                .into_iter()
                .map(|(name, function)| (name, function.fold()))
                .collect(),
        }
    }
}

impl Foldable for hir::Function {
    fn fold(self) -> Self {
        hir::Function {
            name: self.name,
            statements: self.statements,
        }
    }
}

impl Foldable for hir::Expression {
    fn fold(self) -> Self {
        use hir::Expression::*;
        use ir::Binary::*;
        match self {
            Integer(integer) => Integer(integer),
            Label(label) => Label(label),
            Temporary(temporary) => Temporary(temporary),
            Memory(memory) => Memory(Box::new(memory.fold())),
            Sequence(statements, expression) => {
                Sequence(Box::new(statements.fold()), Box::new(expression.fold()))
            }
            Call(name, arguments, returns) => Call(
                Box::new(name.fold()),
                arguments.into_iter().map(Foldable::fold).collect(),
                returns,
            ),
            Binary(binary, left, right) => match (binary, left.fold(), right.fold()) {
                (Add, Integer(l), Integer(r)) => Integer(l + r),
                (Sub, Integer(l), Integer(r)) => Integer(l - r),
                (Mul, Integer(l), Integer(r)) => Integer((l as i128 * r as i128) as i64),
                (Hul, Integer(l), Integer(r)) => Integer(((l as i128 * r as i128) >> 64) as i64),
                (Xor, Integer(l), Integer(r)) => Integer(l ^ r),
                (Ls, Integer(l), Integer(r)) => Integer(l << r),
                (Rs, Integer(l), Integer(r)) => Integer((l as u64 >> r) as i64),
                (ARs, Integer(l), Integer(r)) => Integer(l >> r),
                (Lt, Integer(l), Integer(r)) => Integer(if l < r { 1 } else { 0 }),
                (Le, Integer(l), Integer(r)) => Integer(if l <= r { 1 } else { 0 }),
                (Ge, Integer(l), Integer(r)) => Integer(if l >= r { 1 } else { 0 }),
                (Gt, Integer(l), Integer(r)) => Integer(if l > r { 1 } else { 0 }),
                (Ne, Integer(l), Integer(r)) => Integer(if l != r { 1 } else { 0 }),
                (Eq, Integer(l), Integer(r)) => Integer(if l == r { 1 } else { 0 }),
                (And, Integer(l), Integer(r)) => Integer(if (l & r) & 0b1 == 0b1 { 1 } else { 0 }),
                (Or, Integer(l), Integer(r)) => Integer(if (l | r) & 0b1 == 0b1 { 1 } else { 0 }),
                (Div, Integer(l), Integer(r)) if r != 0 => Integer(l / r),
                (Mod, Integer(l), Integer(r)) if r != 0 => Integer(l % r),

                (Add, Integer(0), Temporary(t))
                | (Add, Temporary(t), Integer(0))
                | (Sub, Temporary(t), Integer(0))
                | (Mul, Temporary(t), Integer(1))
                | (Mul, Integer(1), Temporary(t))
                | (Div, Temporary(t), Integer(1))
                | (Ls, Temporary(t), Integer(0))
                | (Rs, Temporary(t), Integer(0))
                | (ARs, Temporary(t), Integer(0)) => Temporary(t),

                (Add, Integer(0), Label(l))
                | (Add, Label(l), Integer(0))
                | (Sub, Label(l), Integer(0)) => Label(l),

                (Mul, Temporary(_), Integer(0))
                | (Mul, Integer(0), Temporary(_))
                | (Hul, Temporary(_), Integer(0))
                | (Hul, Integer(0), Temporary(_))
                | (Mod, Temporary(_), Integer(1)) => Integer(0),

                (Lt, Temporary(t), Temporary(u))
                | (Gt, Temporary(t), Temporary(u))
                | (Ne, Temporary(t), Temporary(u))
                | (Sub, Temporary(t), Temporary(u))
                    if t == u =>
                {
                    Integer(0)
                }

                (Le, Temporary(t), Temporary(u))
                | (Ge, Temporary(t), Temporary(u))
                | (Eq, Temporary(t), Temporary(u))
                | (Div, Temporary(t), Temporary(u))
                    if t == u =>
                {
                    Integer(1)
                }

                (b, l, r) => Binary(b, Box::new(l), Box::new(r)),
            },
        }
    }
}

impl Foldable for hir::Statement {
    fn fold(self) -> Self {
        use hir::Statement::*;
        match self {
            Expression(expression) => Expression(expression.fold()),
            Jump(expression) => Jump(expression.fold()),
            Label(label) => Label(label),
            Move(into, from) => Move(into.fold(), from.fold()),
            Return(expressions) => Return(expressions.into_iter().map(Foldable::fold).collect()),
            Sequence(statements) => Sequence(statements.into_iter().map(Foldable::fold).collect()),
            CJump(condition, r#true, r#false) => match condition.fold() {
                hir::Expression::Integer(1) => Jump(hir::Expression::Label(r#true)),
                hir::Expression::Integer(0) => Jump(hir::Expression::Label(r#false)),
                expression => CJump(expression, r#true, r#false),
            },
        }
    }
}

impl Foldable for ir::Unit<lir::Function> {
    fn fold(self) -> Self {
        ir::Unit {
            name: self.name,
            data: self.data,
            functions: self
                .functions
                .into_iter()
                .map(|(name, function)| (name, function.fold()))
                .collect(),
        }
    }
}

impl Foldable for lir::Function {
    fn fold(self) -> Self {
        lir::Function {
            name: self.name,
            statements: self.statements.into_iter().map(Foldable::fold).collect(),
        }
    }
}

impl Foldable for lir::Expression {
    fn fold(self) -> Self {
        use ir::Binary::*;
        use lir::Expression::*;
        match self {
            Integer(integer) => Integer(integer),
            Memory(memory) => Memory(Box::new(memory.fold())),
            Label(label) => Label(label),
            Temporary(temporary) => Temporary(temporary),
            Binary(binary, left, right) => match (binary, left.fold(), right.fold()) {
                (Add, Integer(l), Integer(r)) => Integer(l + r),
                (Sub, Integer(l), Integer(r)) => Integer(l - r),
                (Mul, Integer(l), Integer(r)) => Integer((l as i128 * r as i128) as i64),
                (Hul, Integer(l), Integer(r)) => Integer(((l as i128 * r as i128) >> 64) as i64),
                (Xor, Integer(l), Integer(r)) => Integer(l ^ r),
                (Ls, Integer(l), Integer(r)) => Integer(l << r),
                (Rs, Integer(l), Integer(r)) => Integer((l as u64 >> r) as i64),
                (ARs, Integer(l), Integer(r)) => Integer(l >> r),
                (Lt, Integer(l), Integer(r)) => Integer(if l < r { 1 } else { 0 }),
                (Le, Integer(l), Integer(r)) => Integer(if l <= r { 1 } else { 0 }),
                (Ge, Integer(l), Integer(r)) => Integer(if l >= r { 1 } else { 0 }),
                (Gt, Integer(l), Integer(r)) => Integer(if l > r { 1 } else { 0 }),
                (Ne, Integer(l), Integer(r)) => Integer(if l != r { 1 } else { 0 }),
                (Eq, Integer(l), Integer(r)) => Integer(if l == r { 1 } else { 0 }),
                (And, Integer(l), Integer(r)) => Integer(if l & r == 1 { 1 } else { 0 }),
                (Or, Integer(l), Integer(r)) => Integer(if l | r == 1 { 1 } else { 0 }),
                (Div, Integer(l), Integer(r)) if r != 0 => Integer(l / r),
                (Mod, Integer(l), Integer(r)) if r != 0 => Integer(l % r),

                (Add, Integer(0), e)
                | (Add, e, Integer(0))
                | (Sub, e, Integer(0))
                | (Mul, e, Integer(1))
                | (Mul, Integer(1), e)
                | (Div, e, Integer(1))
                | (Ls, e, Integer(0))
                | (Rs, e, Integer(0))
                | (ARs, e, Integer(0)) => e,

                (Mul, _, Integer(0))
                | (Mul, Integer(0), _)
                | (Hul, _, Integer(0))
                | (Hul, Integer(0), _)
                | (Mod, _, Integer(1)) => Integer(0),

                (Lt, t, u) | (Gt, t, u) | (Ne, t, u) | (Sub, t, u) if t == u => Integer(0),
                (Le, t, u) | (Ge, t, u) | (Eq, t, u) | (Div, t, u) if t == u => Integer(1),

                (b, l, r) => Binary(b, Box::new(l), Box::new(r)),
            },
        }
    }
}

impl Foldable for lir::Statement {
    fn fold(self) -> Self {
        use lir::Statement::*;
        match self {
            Jump(expression) => Jump(expression.fold()),
            Call(function, expressions, returns) => Call(
                function.fold(),
                expressions.into_iter().map(Foldable::fold).collect(),
                returns,
            ),
            Move(into, from) => Move(into.fold(), from.fold()),
            Return(expressions) => Return(expressions.into_iter().map(Foldable::fold).collect()),
            Label(label) => Label(label),
            CJump(condition, r#true, r#false) => match condition.fold() {
                lir::Expression::Integer(1) => Jump(lir::Expression::Label(r#true)),
                lir::Expression::Integer(0) => Jump(lir::Expression::Label(r#false)),
                condition => CJump(condition, r#true, r#false),
            },
        }
    }
}
