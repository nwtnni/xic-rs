use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::analyze::Earliest;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;

pub struct PostponableExpressions<T: lir::Target> {
    pub(super) earliest: BTreeMap<Label, Vec<BTreeSet<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for PostponableExpressions<T> {
    const BACKWARD: bool = false;

    type Data = BTreeSet<lir::Expression>;

    fn new() -> Self {
        unreachable!()
    }

    fn new_with_metadata(cfg: &Cfg<lir::Function<T>>) -> Self {
        Self {
            earliest: Earliest::new_with_metadata(cfg).into_inner(),
            marker: PhantomData,
        }
    }

    fn default(&self) -> Self::Data {
        BTreeSet::new()
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
                AnticipatedExpressions::remove(output, left);
                AnticipatedExpressions::remove(output, right);
            }
            lir::Statement::Call(function, arguments, _) => {
                AnticipatedExpressions::remove(output, function);
                for argument in arguments {
                    AnticipatedExpressions::remove(output, argument);
                }
            }
            lir::Statement::Move {
                destination: _,
                source,
            } => {
                AnticipatedExpressions::remove(output, source);
            }
            lir::Statement::Return(returns) => {
                for r#return in returns {
                    AnticipatedExpressions::remove(output, r#return);
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
