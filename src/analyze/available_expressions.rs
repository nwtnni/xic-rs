use std::marker::PhantomData;

use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::AnticipatedExpressions;
use crate::data::lir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::Map;
use crate::Set;

pub struct AvailableExpressions<T: lir::Target> {
    pub(super) anticipated: Map<Label, Vec<Set<lir::Expression>>>,
    marker: PhantomData<T>,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for AvailableExpressions<T> {
    const BACKWARD: bool = false;

    type Data = Set<lir::Expression>;

    fn new() -> Self {
        unreachable!()
    }

    fn new_with_metadata(cfg: &crate::cfg::Cfg<lir::Function<T>>) -> Self {
        let mut solution = analyze::<AnticipatedExpressions, _>(cfg);
        let mut anticipated = Map::default();

        for (label, statements) in cfg.blocks() {
            let mut output = solution.inputs.remove(label).unwrap();
            let mut outputs = vec![output.clone()];

            for (index, statement) in statements.iter().enumerate().rev() {
                solution
                    .analysis
                    .transfer_with_metadata(label, index, statement, &mut output);
                outputs.push(output.clone());
            }

            outputs.reverse();
            anticipated.insert(*label, outputs);
        }

        Self {
            anticipated,
            marker: PhantomData,
        }
    }

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
        output.extend(self.anticipated[label][index].iter().cloned());

        match statement {
            lir::Statement::Jump(_)
            | lir::Statement::CJump { .. }
            | lir::Statement::Label(_)
            | lir::Statement::Return(_) => (),
            lir::Statement::Call(_, _, returns) => {
                for r#return in 0..*returns {
                    Self::remove(
                        output,
                        &lir::Expression::Temporary(Temporary::Return(r#return)),
                    );
                }

                // Conservatively assume all memory is overwritten by call
                Self::remove(
                    output,
                    &lir::Expression::Memory(Box::new(lir::Expression::from(Temporary::Fixed(
                        symbol::intern_static("clobber"),
                    )))),
                )
            }
            lir::Statement::Move {
                destination,
                source: _,
            } => {
                Self::remove(output, destination);
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
        );
    }
}

impl<T: lir::Target> AvailableExpressions<T> {
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
            lir::Expression::Memory(address) => {
                // Conservatively clobber all memory expressions
                matches!(killed, lir::Expression::Memory(_)) || Self::contains(&*address, killed)
            }
            lir::Expression::Binary(_, left, right) => {
                Self::contains(&*left, killed) || Self::contains(&*right, killed)
            }
        }
    }
}
