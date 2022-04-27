use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;

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
            Immediate(immediate) => Immediate(immediate),
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
            Binary(binary, left, right) => {
                const ZERO: hir::Expression =
                    hir::Expression::Immediate(operand::Immediate::Constant(0));

                const ONE: hir::Expression =
                    hir::Expression::Immediate(operand::Immediate::Constant(1));

                use operand::Immediate::Constant;
                use operand::Immediate::Label;

                match (binary, left.fold(), right.fold()) {
                    (_, Immediate(Constant(left)), Immediate(Constant(right))) => {
                        #[rustfmt::skip]
                        let value = match binary {
                            Add => left + right,
                            Sub => left - right,
                            Mul => left * right,
                            Hul => ((left as i128 * right as i128) >> 64) as i64,
                            Xor => left ^ right,
                            Ls => left << right,
                            Rs => (left as u64 >> right) as i64,
                            ARs => left >> right,
                            Lt => if left < right { 1 } else { 0 },
                            Le => if left <= right { 1 } else { 0 },
                            Ge => if left >= right { 1 } else { 0 },
                            Gt => if left > right { 1 } else { 0 },
                            Ne => if left != right { 1 } else { 0 },
                            Eq => if left == right { 1 } else { 0 },
                            And => if (left & right) & 1 > 0 { 1 } else { 0 },
                            Or => if (left | right) & 1 > 0 { 1 } else { 0 },
                            Div => left / right,
                            Mod => left % right,
                        };

                        hir::Expression::from(value)
                    }

                    (Add, ZERO, Temporary(temporary))
                    | (Add, Temporary(temporary), ZERO)
                    | (Sub, Temporary(temporary), ZERO)
                    | (Mul, Temporary(temporary), ONE)
                    | (Mul, ONE, Temporary(temporary))
                    | (Div, Temporary(temporary), ONE)
                    | (Ls, Temporary(temporary), ZERO)
                    | (Rs, Temporary(temporary), ZERO)
                    | (ARs, Temporary(temporary), ZERO) => Temporary(temporary),

                    (Add, ZERO, Immediate(Label(label)))
                    | (Add, Immediate(Label(label)), ZERO)
                    | (Sub, Immediate(Label(label)), ZERO) => Immediate(Label(label)),

                    (Mul, Temporary(_), ZERO)
                    | (Mul, ZERO, Temporary(_))
                    | (Hul, Temporary(_), ZERO)
                    | (Hul, ZERO, Temporary(_))
                    | (Mod, Temporary(_), ONE) => ZERO,

                    (Lt, Temporary(left), Temporary(right))
                    | (Gt, Temporary(left), Temporary(right))
                    | (Ne, Temporary(left), Temporary(right))
                    | (Sub, Temporary(left), Temporary(right))
                        if left == right =>
                    {
                        ZERO
                    }

                    (Le, Temporary(left), Temporary(right))
                    | (Ge, Temporary(left), Temporary(right))
                    | (Eq, Temporary(left), Temporary(right))
                    | (Div, Temporary(left), Temporary(right))
                        if left == right =>
                    {
                        ONE
                    }

                    (binary, left, right) => Binary(binary, Box::new(left), Box::new(right)),
                }
            }
        }
    }
}

impl Foldable for hir::Statement {
    fn fold(self) -> Self {
        use hir::Statement::*;
        match self {
            Expression(expression) => Expression(expression.fold()),
            Jump(label) => Jump(label),
            Label(label) => Label(label),
            Move {
                destination,
                source,
            } => Move {
                destination: destination.fold(),
                source: source.fold(),
            },
            Return => Return,
            Sequence(statements) => Sequence(statements.into_iter().map(Foldable::fold).collect()),
            CJump {
                condition,
                r#true,
                r#false,
            } => match condition.fold() {
                hir::Expression::Immediate(operand::Immediate::Constant(1)) => Jump(r#true),
                hir::Expression::Immediate(operand::Immediate::Constant(0)) => Jump(r#false),
                condition => CJump {
                    condition,
                    r#true,
                    r#false,
                },
            },
        }
    }
}

impl<T: lir::Target> Foldable for ir::Unit<lir::Function<T>> {
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

impl<T: lir::Target> Foldable for lir::Function<T> {
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

impl<T: lir::Target> Foldable for lir::Statement<T> {
    fn fold(self) -> Self {
        use lir::Statement::*;
        match self {
            Jump(label) => Jump(label),
            Call(function, expressions, returns) => Call(
                function.fold(),
                expressions.into_iter().map(Foldable::fold).collect(),
                returns,
            ),
            Move {
                destination,
                source,
            } => Move {
                destination: destination.fold(),
                source: source.fold(),
            },
            Return => Return,
            Label(label) => Label(label),
            CJump {
                condition,
                r#true,
                r#false,
            } => match (condition.fold(), r#false.label()) {
                (lir::Expression::Integer(1), _) => Jump(r#true),
                (lir::Expression::Integer(0), Some(label)) => Jump(*label),
                (condition, _) => CJump {
                    condition,
                    r#true,
                    r#false,
                },
            },
        }
    }
}
