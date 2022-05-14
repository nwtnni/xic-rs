use std::collections::BTreeMap;

use crate::analyze::Analysis;
use crate::cfg::Edge;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Temporary;
use crate::optimize;

pub struct ConditionalConstantPropagation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    pub(crate) reachable: Reachable,
    pub(crate) constants: BTreeMap<Temporary, Constant>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reachable {
    Linear(bool),
    Branch(bool, bool),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Constant {
    Defined(Immediate),
    Overdefined,
}

impl<T: lir::Target> Analysis<lir::Function<T>> for ConditionalConstantPropagation {
    const BACKWARD: bool = false;

    type Data = Data;

    fn new() -> Self {
        ConditionalConstantPropagation
    }

    fn default(&self) -> Self::Data {
        Data {
            reachable: Reachable::Linear(true),
            constants: BTreeMap::new(),
        }
    }

    fn transfer(&self, statement: &lir::Statement<T>, output: &mut Self::Data) {
        match statement {
            lir::Statement::Jump(_) => (),
            lir::Statement::Label(_) => (),
            lir::Statement::CJump {
                condition,
                left,
                right,
                r#true: _,
                r#false: _,
            } => {
                // FIXME: collect symbolic constraints based on branch
                output.reachable = match output.evaluate_condition(condition, left, right) {
                    None => Reachable::Branch(true, true),
                    Some(true) => Reachable::Branch(true, false),
                    Some(false) => Reachable::Branch(false, true),
                }
            }
            lir::Statement::Call(_, _, _) => todo!(),
            lir::Statement::Move {
                destination: lir::Expression::Temporary(temporary),
                source,
            } => {
                if let Some(immediate) = output.evaluate_expression(source) {
                    output
                        .constants
                        .entry(*temporary)
                        .and_modify(|constant| match constant {
                            Constant::Overdefined => (),
                            Constant::Defined(constant) if *constant == immediate => (),
                            Constant::Defined(_) => *constant = Constant::Overdefined,
                        })
                        .or_insert(Constant::Defined(immediate));
                }
            }
            lir::Statement::Move { .. } => (),

            // Note: normally, we would mark anything after a return as unreachable,
            // but we desugar returns into move + jump statements when we tile assembly.
            lir::Statement::Return(_) => (),
        }
    }

    fn merge<'a, I>(&self, _: I, _: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        unreachable!()
    }

    fn merge_with_metadata<'a, I>(&self, mut outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = (&'a Edge, Option<&'a Self::Data>)>,
        Self::Data: 'a,
    {
        match outputs.next() {
            None | Some((_, None)) => return,
            Some((edge, Some(output))) => {
                input.reachable = match (edge, output.reachable) {
                    (Edge::Unconditional, Reachable::Linear(reachable))
                    | (Edge::Conditional(true), Reachable::Branch(reachable, _))
                    | (Edge::Conditional(false), Reachable::Branch(_, reachable)) => {
                        Reachable::Linear(reachable)
                    }
                    _ => unreachable!(),
                };

                input.constants.clear();
                input.constants.extend(
                    output
                        .constants
                        .iter()
                        .map(|(temporary, constant)| (*temporary, *constant)),
                );
            }
        }

        for (edge, output) in
            outputs.filter_map(|(edge, output)| output.map(move |output| (edge, output)))
        {
            match (edge, output.reachable, &mut input.reachable) {
                (Edge::Unconditional, Reachable::Linear(from), Reachable::Linear(to))
                | (Edge::Conditional(true), Reachable::Branch(from, _), Reachable::Linear(to))
                | (Edge::Conditional(false), Reachable::Branch(_, from), Reachable::Linear(to)) => {
                    *to |= from;
                }
                _ => unreachable!(),
            }

            input.constants.retain(
                |temporary, old| match (old, output.constants.get(temporary)) {
                    (Constant::Defined(_), None) => true,
                    (Constant::Defined(old), Some(Constant::Defined(new))) if old == new => true,
                    (
                        old @ Constant::Defined(_),
                        Some(Constant::Defined(_) | Constant::Overdefined),
                    ) => {
                        *old = Constant::Overdefined;
                        true
                    }
                    (
                        Constant::Overdefined,
                        Some(Constant::Defined(_) | Constant::Overdefined) | None,
                    ) => true,
                },
            );
        }
    }
}

impl Data {
    pub fn evaluate_condition(
        &self,
        condition: &ir::Condition,
        left: &lir::Expression,
        right: &lir::Expression,
    ) -> Option<bool> {
        let (left, right) = match (
            self.evaluate_expression(left)?,
            self.evaluate_expression(right)?,
        ) {
            (Immediate::Integer(left), Immediate::Integer(right)) => (left, right),
            _ => return None,
        };

        Some(optimize::fold_condition(*condition, left, right))
    }

    pub fn evaluate_expression(&self, expression: &lir::Expression) -> Option<Immediate> {
        match expression {
            lir::Expression::Argument(_)
            | lir::Expression::Return(_)
            // Conservatively mark all memory as unknown for now.
            | lir::Expression::Memory(_) => None,
            lir::Expression::Immediate(immediate) => Some(*immediate),
            lir::Expression::Temporary(temporary) => match self.constants.get(temporary)? {
                Constant::Defined(immediate) => Some(*immediate),
                Constant::Overdefined => None,
            },
            lir::Expression::Binary(binary, left, right) => {
                // FIXME: handle labels with offsets at some point
                let (left, right) = match (self.evaluate_expression(left)?, self.evaluate_expression(right)?) {
                    (Immediate::Integer(left), Immediate::Integer(right)) => (left, right),
                    _ => return None,
                };

                // FIXME: handle division by 0, algebraic identities
                Some(Immediate::Integer(optimize::fold_binary(*binary, left, right)))
            }
        }
    }
}
