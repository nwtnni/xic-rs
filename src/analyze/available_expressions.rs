use std::collections::BTreeSet;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::analyze::Solution;
use crate::data::lir;
use crate::data::operand::Label;

pub struct AvailableExpressions<T: lir::Target> {
    anticipated: Solution<AnticipatedExpressions, lir::Function<T>>,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for AvailableExpressions<T> {
    const BACKWARD: bool = false;

    type Data = BTreeSet<lir::Expression>;

    fn new() -> Self {
        unreachable!()
    }

    fn new_with_metadata(cfg: &crate::cfg::Cfg<lir::Function<T>>) -> Self {
        Self {
            anticipated: analyze(cfg),
        }
    }

    fn default(&self) -> Self::Data {
        unreachable!()
    }

    fn default_with_metadata(&self, label: &Label) -> Self::Data {
        self.anticipated.outputs[label].clone()
    }

    fn transfer(&self, statement: &lir::Statement<T>, output: &mut Self::Data) {
        match statement {
            lir::Statement::Jump(_)
            | lir::Statement::CJump { .. }
            | lir::Statement::Label(_)
            | lir::Statement::Return(_) => (),
            lir::Statement::Call(_, _, returns) => {
                for r#return in 0..*returns {
                    AnticipatedExpressions::remove(output, &lir::Expression::Return(r#return));
                }
            }
            lir::Statement::Move {
                destination,
                source: _,
            } => {
                AnticipatedExpressions::remove(output, destination);
            }
        }
    }

    fn merge<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        <AnticipatedExpressions as Analysis<lir::Function<T>>>::merge(
            &self.anticipated.analysis,
            outputs,
            input,
        );
    }
}
