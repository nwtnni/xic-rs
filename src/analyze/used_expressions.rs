use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::analyze::Latest;
use crate::cfg::Cfg;
use crate::data::lir;
use crate::data::operand::Label;

pub struct UsedExpressions<T: lir::Target> {
    latest: BTreeMap<Label, Vec<BTreeSet<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for UsedExpressions<T> {
    const BACKWARD: bool = true;

    type Data = BTreeSet<lir::Expression>;

    fn new() -> Self {
        todo!()
    }

    fn new_with_metadata(cfg: &Cfg<lir::Function<T>>) -> Self {
        Self {
            latest: Latest::new_with_metadata(cfg).into_inner(),
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
                destination: _,
                source,
            } => {
                AnticipatedExpressions::insert(output, source);
            }
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
