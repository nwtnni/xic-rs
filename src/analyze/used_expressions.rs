use std::marker::PhantomData;

use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::analyze::Latest;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;
use crate::Map;
use crate::Set;

pub struct UsedExpressions<T: lir::Target> {
    pub(crate) latest: Map<Label, Vec<Set<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> UsedExpressions<T> {
    pub fn new(cfg: &Cfg<lir::Function<T>>) -> Self {
        Self {
            latest: Latest::new(cfg).into_inner(),
            marker: PhantomData,
        }
    }
}

impl<T: lir::Target> Analysis<lir::Function<T>> for UsedExpressions<T> {
    const BACKWARD: bool = true;

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
        match statement {
            lir::Statement::Jump(_) | lir::Statement::Label(_) => (),
            lir::Statement::CJump {
                condition: _,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                AnticipatedExpressions::insert(output, left);
                AnticipatedExpressions::insert(output, right);
            }
            lir::Statement::Call(function, arguments, _) => {
                AnticipatedExpressions::insert(output, function);
                for argument in arguments {
                    AnticipatedExpressions::insert(output, argument);
                }
            }
            lir::Statement::Move {
                destination: lir::Expression::Temporary(_),
                source,
            } => {
                AnticipatedExpressions::insert(output, source);
            }
            lir::Statement::Move {
                destination: lir::Expression::Memory(address),
                source,
            } => {
                AnticipatedExpressions::insert(output, address);
                AnticipatedExpressions::insert(output, source);
            }
            lir::Statement::Move { .. } => unreachable!(),
            lir::Statement::Return(returns) => {
                for r#return in returns {
                    AnticipatedExpressions::insert(output, r#return);
                }
            }
        }

        output.retain(|expression| !self.latest[label][index].contains(expression));
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        input.clear();
        input.extend(outputs.flatten().flatten().cloned());
    }
}
