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

                match (binary, left.fold(), right.fold()) {
                    (
                        _,
                        Immediate(operand::Immediate::Constant(left)),
                        Immediate(operand::Immediate::Constant(right)),
                    ) => hir::Expression::from(fold_binary(binary, left, right)),

                    (Add, ZERO, Temporary(temporary))
                    | (Add, Temporary(temporary), ZERO)
                    | (Sub, Temporary(temporary), ZERO)
                    | (Mul, Temporary(temporary), ONE)
                    | (Mul, ONE, Temporary(temporary))
                    | (Div, Temporary(temporary), ONE)
                    | (Ls, Temporary(temporary), ZERO)
                    | (Rs, Temporary(temporary), ZERO)
                    | (ARs, Temporary(temporary), ZERO) => Temporary(temporary),

                    (Add, ZERO, Immediate(operand::Immediate::Label(label)))
                    | (Add, Immediate(operand::Immediate::Label(label)), ZERO)
                    | (Sub, Immediate(operand::Immediate::Label(label)), ZERO) => {
                        Immediate(operand::Immediate::Label(label))
                    }

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
            Immediate(immediate) => Immediate(immediate),
            Memory(memory) => Memory(Box::new(memory.fold())),
            Temporary(temporary) => Temporary(temporary),
            Binary(binary, left, right) => {
                const ZERO: lir::Expression =
                    lir::Expression::Immediate(operand::Immediate::Constant(0));

                const ONE: lir::Expression =
                    lir::Expression::Immediate(operand::Immediate::Constant(1));

                match (binary, left.fold(), right.fold()) {
                    (
                        _,
                        Immediate(operand::Immediate::Constant(left)),
                        Immediate(operand::Immediate::Constant(right)),
                    ) => Immediate(operand::Immediate::Constant(fold_binary(
                        binary, left, right,
                    ))),

                    (Add, ZERO, expression)
                    | (Add, expression, ZERO)
                    | (Sub, expression, ZERO)
                    | (Mul, expression, ONE)
                    | (Mul, ONE, expression)
                    | (Div, expression, ONE)
                    | (Ls, expression, ZERO)
                    | (Rs, expression, ZERO)
                    | (ARs, expression, ZERO) => expression,

                    (Mul, _, ZERO)
                    | (Mul, ZERO, _)
                    | (Hul, _, ZERO)
                    | (Hul, ZERO, _)
                    | (Mod, _, ONE) => ZERO,

                    (Lt, left, right)
                    | (Gt, left, right)
                    | (Ne, left, right)
                    | (Sub, left, right)
                        if left == right =>
                    {
                        ZERO
                    }

                    (Le, left, right)
                    | (Ge, left, right)
                    | (Eq, left, right)
                    | (Div, left, right)
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
                (lir::Expression::Immediate(operand::Immediate::Constant(1)), _) => Jump(r#true),
                (lir::Expression::Immediate(operand::Immediate::Constant(0)), Some(label)) => {
                    Jump(*label)
                }
                (condition, _) => CJump {
                    condition,
                    r#true,
                    r#false,
                },
            },
        }
    }
}

#[rustfmt::skip]
fn fold_binary(binary: ir::Binary, left: i64, right: i64) -> i64 {
    match binary {
        ir::Binary::Add => left + right,
        ir::Binary::Sub => left - right,
        ir::Binary::Mul => left * right,
        ir::Binary::Hul => ((left as i128 * right as i128) >> 64) as i64,
        ir::Binary::Xor => left ^ right,
        ir::Binary::Ls => left << right,
        ir::Binary::Rs => (left as u64 >> right) as i64,
        ir::Binary::ARs => left >> right,
        ir::Binary::Lt => if left < right { 1 } else { 0 },
        ir::Binary::Le => if left <= right { 1 } else { 0 },
        ir::Binary::Ge => if left >= right { 1 } else { 0 },
        ir::Binary::Gt => if left > right { 1 } else { 0 },
        ir::Binary::Ne => if left != right { 1 } else { 0 },
        ir::Binary::Eq => if left == right { 1 } else { 0 },
        ir::Binary::And => if (left & right) & 1 > 0 { 1 } else { 0 },
        ir::Binary::Or => if (left | right) & 1 > 0 { 1 } else { 0 },
        ir::Binary::Div => left / right,
        ir::Binary::Mod => left % right,
    }
}
