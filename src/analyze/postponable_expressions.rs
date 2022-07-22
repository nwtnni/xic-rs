use std::marker::PhantomData;

use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::analyze::Earliest;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;
use crate::Map;
use crate::Set;

pub struct PostponableExpressions<T: lir::Target> {
    pub(super) earliest: Map<Label, Vec<Set<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> PostponableExpressions<T> {
    pub fn new(cfg: &Cfg<lir::Function<T>>) -> Self {
        Self {
            earliest: Earliest::new(cfg).into_inner(),
            marker: PhantomData,
        }
    }
}

impl<T: lir::Target> Analysis<lir::Function<T>> for PostponableExpressions<T> {
    const BACKWARD: bool = false;

    type Data = Set<lir::Expression>;

    fn default(&self) -> Self::Data {
        Set::default()
    }

    fn transfer(&self, _: &lir::Statement<T>, _: &mut Self::Data) {
        unreachable!()
    }

    fn transfer_with_metadata(
        &self,
        label: &Label,
        index: usize,
        statement: &lir::Statement<T>,
        output: &mut Self::Data,
    ) {
        output.extend(self.earliest[label][index].iter().cloned());

        match statement {
            lir::Statement::Jump(_) | lir::Statement::Label(_) => (),
            lir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                Self::remove(output, left);
                Self::remove(output, right);
            }
            lir::Statement::Call(function, arguments, _) => {
                Self::remove(output, function);
                for argument in arguments {
                    Self::remove(output, argument);
                }
            }
            lir::Statement::Move {
                destination: lir::Expression::Temporary(_),
                source,
            } => {
                Self::remove(output, source);
            }
            lir::Statement::Move {
                destination: lir::Expression::Memory(address),
                source,
            } => {
                Self::remove(output, address);
                Self::remove(output, source);
            }
            lir::Statement::Move { .. } => unreachable!(),
            lir::Statement::Return(returns) => {
                for r#return in returns {
                    Self::remove(output, r#return);
                }
            }
        }
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        <AnticipatedExpressions as Analysis<lir::Function<T>>>::merge(
            &AnticipatedExpressions,
            outputs,
            input,
        )
    }
}

impl<T: lir::Target> PostponableExpressions<T> {
    pub(super) fn remove(output: &mut Set<lir::Expression>, kill: &lir::Expression) {
        output.remove(kill);

        match kill {
            lir::Expression::Immediate(_) | lir::Expression::Temporary(_) => (),
            lir::Expression::Memory(address) => Self::remove(output, address),
            lir::Expression::Binary(_, left, right) => {
                Self::remove(output, left);
                Self::remove(output, right);
            }
        }
    }
}
