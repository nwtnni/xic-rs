use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::util;

pub trait Foldable {
    fn fold(self) -> Self;
}

pub fn fold<T: Foldable>(foldable: T) -> T {
    foldable.fold()
}

impl Foldable for hir::Function {
    fn fold(self) -> Self {
        log::info!(
            "[{}] Folding constants in {}...",
            std::any::type_name::<hir::Function>(),
            self.name
        );
        util::time!(
            "[{}] Done folding constants in {}",
            std::any::type_name::<hir::Function>(),
            self.name,
        );

        hir::Function {
            name: self.name,
            statement: self.statement.fold(),
            arguments: self.arguments,
            returns: self.returns,
            visibility: self.visibility,
        }
    }
}

impl<T: lir::Target> Foldable for lir::Function<T> {
    fn fold(self) -> Self {
        log::info!(
            "[{}] Folding constants in {}...",
            std::any::type_name::<lir::Function<T>>(),
            self.name,
        );
        util::time!(
            "[{}] Done folding constants in {}",
            std::any::type_name::<lir::Function<T>>(),
            self.name,
        );

        lir::Function {
            name: self.name,
            statements: self.statements.fold(),
            arguments: self.arguments,
            returns: self.returns,
            visibility: self.visibility,
            enter: self.enter,
            exit: self.exit,
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
            Call(name, arguments, returns) => {
                Call(Box::new(name.fold()), arguments.fold(), returns)
            }
            Binary(binary, left, right) => match (binary, left.fold(), right.fold()) {
                (
                    _,
                    Immediate(operand::Immediate::Integer(left)),
                    Immediate(operand::Immediate::Integer(right)),
                ) => hir::Expression::from(fold_binary(binary, left, right)),

                (Add, hir::ZERO, Temporary(temporary))
                | (Add, Temporary(temporary), hir::ZERO)
                | (Sub, Temporary(temporary), hir::ZERO)
                | (Mul, Temporary(temporary), hir::ONE)
                | (Mul, hir::ONE, Temporary(temporary))
                | (Div, Temporary(temporary), hir::ONE) => Temporary(temporary),

                (Add, hir::ZERO, Immediate(operand::Immediate::Label(label)))
                | (Add, Immediate(operand::Immediate::Label(label)), hir::ZERO)
                | (Sub, Immediate(operand::Immediate::Label(label)), hir::ZERO) => {
                    Immediate(operand::Immediate::Label(label))
                }

                (Mul, Temporary(_), hir::ZERO)
                | (Mul, hir::ZERO, Temporary(_))
                | (Hul, Temporary(_), hir::ZERO)
                | (Hul, hir::ZERO, Temporary(_))
                | (Mod, Temporary(_), hir::ONE) => hir::ZERO,

                (Sub, Temporary(left), Temporary(right)) if left == right => hir::ZERO,
                (Div, Temporary(left), Temporary(right)) if left == right => hir::ONE,

                (binary, left, right) => Binary(binary, Box::new(left), Box::new(right)),
            },
        }
    }
}

impl Foldable for hir::Statement {
    fn fold(self) -> Self {
        use hir::Expression::Immediate;
        use hir::Expression::Temporary;
        use hir::Statement::*;
        use ir::Condition::*;
        use operand::Immediate::Integer;

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
            Return(returns) => Return(returns.fold()),
            Sequence(statements) => Sequence(statements.fold()),
            CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => match (condition, left.fold(), right.fold()) {
                (_, Immediate(Integer(left)), Immediate(Integer(right))) => {
                    if fold_condition(condition, left, right) {
                        Jump(r#true)
                    } else {
                        Jump(r#false)
                    }
                }

                (Lt, Temporary(left), Temporary(right))
                | (Gt, Temporary(left), Temporary(right))
                | (Ne, Temporary(left), Temporary(right))
                    if left == right =>
                {
                    Jump(r#true)
                }

                (Le, Temporary(left), Temporary(right))
                | (Ge, Temporary(left), Temporary(right))
                | (Eq, Temporary(left), Temporary(right))
                    if left == right =>
                {
                    Jump(r#false)
                }

                (condition, left, right) => CJump {
                    condition,
                    left,
                    right,
                    r#true,
                    r#false,
                },
            },
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
            Binary(binary, left, right) => match (binary, left.fold(), right.fold()) {
                (
                    _,
                    Immediate(operand::Immediate::Integer(left)),
                    Immediate(operand::Immediate::Integer(right)),
                ) => Immediate(operand::Immediate::Integer(fold_binary(
                    binary, left, right,
                ))),

                (Add, lir::ZERO, expression)
                | (Add, expression, lir::ZERO)
                | (Sub, expression, lir::ZERO)
                | (Mul, expression, lir::ONE)
                | (Mul, lir::ONE, expression)
                | (Div, expression, lir::ONE) => expression,

                (Mul, _, lir::ZERO)
                | (Mul, lir::ZERO, _)
                | (Hul, _, lir::ZERO)
                | (Hul, lir::ZERO, _)
                | (Mod, _, lir::ONE) => lir::ZERO,

                (Sub, left, right) if left == right => lir::ZERO,
                (Div, left, right) if left == right => lir::ONE,

                (binary, left, right) => Binary(binary, Box::new(left), Box::new(right)),
            },
        }
    }
}

impl<T: lir::Target> Foldable for lir::Statement<T> {
    fn fold(self) -> Self {
        use ir::Condition::*;
        use lir::Expression::Immediate;
        use lir::Statement::*;
        use operand::Immediate::Integer;

        match self {
            Jump(label) => Jump(label),
            Call(function, expressions, returns) => {
                Call(function.fold(), expressions.fold(), returns)
            }
            Move {
                destination,
                source,
            } => Move {
                destination: destination.fold(),
                source: source.fold(),
            },
            Return(returns) => Return(returns.fold()),
            Label(label) => Label(label),
            CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => match (condition, left.fold(), right.fold()) {
                (condition, Immediate(Integer(left)), Immediate(Integer(right))) => {
                    if fold_condition(condition, left, right) {
                        Jump(r#true)
                    } else if let Some(r#false) = r#false.target() {
                        Jump(*r#false)
                    } else {
                        Label(operand::Label::fresh("nop"))
                    }
                }

                (Le, left, right) | (Ge, left, right) | (Eq, left, right) if left == right => {
                    Jump(r#true)
                }
                (Lt, left, right) | (Gt, left, right) | (Ne, left, right) if left == right => {
                    match r#false.target() {
                        Some(r#false) => Jump(*r#false),
                        None => Label(operand::Label::fresh("nop")),
                    }
                }

                (condition, left, right) => CJump {
                    condition,
                    left,
                    right,
                    r#true,
                    r#false,
                },
            },
        }
    }
}

impl<T: Foldable> Foldable for Vec<T> {
    fn fold(self) -> Self {
        self.into_iter().map(Foldable::fold).collect()
    }
}

pub fn fold_binary(binary: ir::Binary, left: i64, right: i64) -> i64 {
    match binary {
        ir::Binary::Add => left.wrapping_add(right),
        ir::Binary::Sub => left.wrapping_sub(right),
        ir::Binary::Mul => left.wrapping_mul(right),
        ir::Binary::Hul => ((left as i128 * right as i128) >> 64) as i64,
        ir::Binary::Xor => left ^ right,
        #[rustfmt::skip]
        ir::Binary::And => if (left & right) & 1 > 0 { 1 } else { 0 },
        #[rustfmt::skip]
        ir::Binary::Or => if (left | right) & 1 > 0 { 1 } else { 0 },
        ir::Binary::Div => left / right,
        ir::Binary::Mod => left % right,
    }
}

pub fn fold_condition(condition: ir::Condition, left: i64, right: i64) -> bool {
    match condition {
        ir::Condition::Lt => left < right,
        ir::Condition::Le => left <= right,
        ir::Condition::Ge => left >= right,
        ir::Condition::Gt => left > right,
        ir::Condition::Ne => left != right,
        ir::Condition::Eq => left == right,
        ir::Condition::Ae => left as u64 >= right as u64,
    }
}
