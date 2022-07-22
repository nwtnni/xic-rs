use crate::analyze::Analysis;
use crate::data::lir;
use crate::Set;

#[derive(Default)]
pub struct AnticipatedExpressions;

impl<T: lir::Target> Analysis<lir::Function<T>> for AnticipatedExpressions {
    const BACKWARD: bool = true;

    type Data = Set<lir::Expression>;

    fn default(&self) -> Self::Data {
        Set::default()
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
                Self::insert(output, left);
                Self::insert(output, right);
            }
            lir::Statement::Call(function, arguments, returns) => {
                for r#return in returns {
                    Self::remove(output, &lir::Expression::Temporary(*r#return));
                }

                Self::insert(output, function);
                for argument in arguments {
                    Self::insert(output, argument);
                }
            }
            lir::Statement::Move {
                destination: destination @ lir::Expression::Temporary(_),
                source,
            } => {
                Self::remove(output, destination);
                Self::insert(output, source);
            }
            lir::Statement::Move {
                destination: destination @ lir::Expression::Memory(address),
                source,
            } => {
                Self::remove(output, destination);
                Self::insert(output, address);
                Self::insert(output, source);
            }
            lir::Statement::Move { .. } => unreachable!(),
            lir::Statement::Return(returns) => {
                for r#return in returns {
                    Self::insert(output, r#return);
                }
            }
        }
    }

    fn merge<'a, I>(&self, mut outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        match outputs.next() {
            None => return,
            Some(output) => {
                input.clear();
                input.extend(output.into_iter().flatten().cloned());
            }
        }

        for output in outputs.flatten() {
            input.retain(|expression| output.contains(expression));
        }
    }
}

impl AnticipatedExpressions {
    pub(super) fn insert(output: &mut Set<lir::Expression>, r#use: &lir::Expression) {
        match r#use {
            lir::Expression::Immediate(_) | lir::Expression::Temporary(_) => (),
            lir::Expression::Memory(address) => {
                Self::insert(output, &*address);
                output.insert(r#use.clone());
            }
            lir::Expression::Binary(_, left, right) => {
                Self::insert(output, &*left);
                Self::insert(output, &*right);
                output.insert(r#use.clone());
            }
        }
    }

    fn remove(output: &mut Set<lir::Expression>, kill: &lir::Expression) {
        output.remove(kill);

        let mut stack = vec![kill.clone()];

        while let Some(killed) = stack.pop() {
            output.retain(|kill| match Self::contains(kill, &killed) {
                false => true,
                true => {
                    stack.push(kill.clone());
                    false
                }
            })
        }
    }

    fn contains(expression: &lir::Expression, killed: &lir::Expression) -> bool {
        match expression {
            lir::Expression::Immediate(_) => false,
            lir::Expression::Temporary(_) => expression == killed,
            lir::Expression::Memory(address) => Self::contains(&*address, killed),
            lir::Expression::Binary(_, left, right) => {
                Self::contains(&*left, killed) || Self::contains(&*right, killed)
            }
        }
    }
}
