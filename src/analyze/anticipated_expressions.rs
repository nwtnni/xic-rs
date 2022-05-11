use std::collections::BTreeSet;

use crate::analyze::Analysis;
use crate::data::lir;

pub struct AnticipatedExpressions;

impl<T: lir::Target> Analysis<lir::Function<T>> for AnticipatedExpressions {
    const BACKWARD: bool = true;

    type Data = BTreeSet<lir::Expression>;

    fn new() -> Self {
        Self
    }

    fn default(&self) -> Self::Data {
        BTreeSet::new()
    }

    fn transfer(&self, statement: &lir::Statement<T>, output: &mut Self::Data) {
        match statement {
            lir::Statement::Jump(_) | lir::Statement::Label(_) => (),
            lir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                output.insert(left.clone());
                output.insert(right.clone());
            }
            lir::Statement::Call(function, arguments, returns) => {
                for r#return in 0..*returns {
                    Self::remove(output, &lir::Expression::Return(r#return));
                }

                output.extend(arguments.iter().cloned());
                output.insert(function.clone());
            }
            lir::Statement::Move {
                destination,
                source,
            } => {
                Self::remove(output, destination);
                output.insert(source.clone());
            }
            lir::Statement::Return(returns) => {
                output.extend(returns.iter().cloned());
            }
        }
    }

    fn merge<'a, I>(&self, mut outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        input.clear();
        input.extend(outputs.next().into_iter().flatten().flatten().cloned());

        let mut stack = Vec::new();

        for output in outputs.flatten() {
            stack.extend(
                input
                    .iter()
                    .filter(|expression| !output.contains(expression))
                    // Could be avoided with https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.drain_filter
                    .cloned(),
            );

            stack.drain(..).for_each(|kill| Self::remove(input, &kill));
        }
    }
}

impl AnticipatedExpressions {
    pub(super) fn remove(output: &mut BTreeSet<lir::Expression>, kill: &lir::Expression) {
        output.remove(kill);

        let mut stack = vec![kill.clone()];

        while let Some(killed) = stack.pop() {
            output.retain(|kill| {
                if !Self::contains(kill, &killed) {
                    return true;
                }

                stack.push(kill.clone());
                false
            })
        }
    }

    fn contains(expression: &lir::Expression, killed: &lir::Expression) -> bool {
        match expression {
            lir::Expression::Argument(_)
            | lir::Expression::Return(_)
            | lir::Expression::Immediate(_)
            | lir::Expression::Temporary(_) => expression == killed,
            lir::Expression::Memory(address) => Self::contains(&*address, killed),
            lir::Expression::Binary(_, left, right) => {
                Self::contains(&*left, killed) || Self::contains(&*right, killed)
            }
        }
    }
}
