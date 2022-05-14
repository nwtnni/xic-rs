use crate::analyze::Analysis;
use crate::cfg::Cfg;
use crate::cfg::Edge;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::optimize;
use crate::Map;

pub struct ConditionalConstantPropagation {
    enter: Label,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    pub(crate) reachable: Reachable,
    pub(crate) constants: Map<Temporary, Constant>,
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
        unreachable!()
    }

    fn new_with_metadata(cfg: &Cfg<lir::Function<T>>) -> Self {
        ConditionalConstantPropagation {
            enter: *cfg.enter(),
        }
    }

    fn default(&self) -> Self::Data {
        unreachable!()
    }

    fn default_with_metadata(&self, label: &Label) -> Self::Data {
        Data {
            reachable: Reachable::Linear(*label == self.enter),
            constants: Map::default(),
        }
    }

    fn transfer(&self, statement: &lir::Statement<T>, output: &mut Self::Data) {
        match output.reachable {
            Reachable::Branch(_, _) => unreachable!(),
            Reachable::Linear(true) => (),
            Reachable::Linear(false) => {
                output.reachable = match statement {
                    lir::Statement::CJump { .. } => Reachable::Branch(false, false),
                    lir::Statement::Jump(_)
                    | lir::Statement::Call(_, _, _)
                    | lir::Statement::Label(_)
                    | lir::Statement::Move { .. }
                    | lir::Statement::Return(_) => Reachable::Linear(false),
                };
                return;
            }
        }

        match statement {
            lir::Statement::Jump(_)
            | lir::Statement::Label(_)
            | lir::Statement::Call(_, _, _)
            | lir::Statement::Move {
                destination: lir::Expression::Memory(_),
                source: _,
            }
            // Note: normally, we would mark anything after a return as unreachable,
            // but we desugar returns into move + jump statements when we tile assembly.
            | lir::Statement::Return(_) => {}
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
                            Constant::Defined(_) => *constant = Constant::Defined(immediate),
                        })
                        .or_insert(Constant::Defined(immediate));
                }
            }
            lir::Statement::Move { .. } => unreachable!(),
        }
    }

    fn merge<'a, I>(&self, _: I, _: &mut Self::Data)
    where
        I: Iterator<Item = Option<&'a Self::Data>>,
        Self::Data: 'a,
    {
        unreachable!()
    }

    fn merge_with_metadata<'a, I>(&self, outputs: I, input: &mut Self::Data)
    where
        I: Iterator<Item = (&'a Edge, Option<&'a Self::Data>)>,
        Self::Data: 'a,
    {
        input.constants.clear();

        let reachable = match &mut input.reachable {
            Reachable::Linear(reachable) => reachable,
            Reachable::Branch(_, _) => unreachable!(),
        };

        for (_, output) in outputs
            .filter_map(|(edge, output)| output.map(move |output| (edge, output)))
            // Ignore merge information from unreachable predecessors
            .filter(|(edge, output)| {
                matches!(
                    (edge, output.reachable),
                    (Edge::Unconditional, Reachable::Linear(true))
                        | (Edge::Conditional(true), Reachable::Branch(true, _))
                        | (Edge::Conditional(false), Reachable::Branch(_, true))
                )
            })
        {
            *reachable = true;

            // Defined in `input` and `output`
            input.constants.retain(
                |temporary, old| match (old, output.constants.get(temporary)) {
                    (Constant::Defined(_), None) => true,
                    (Constant::Defined(old), Some(Constant::Defined(new))) if old == new => true,
                    (old, _) => {
                        *old = Constant::Overdefined;
                        true
                    }
                },
            );

            // Defined in `output`
            for (temporary, constant) in &output.constants {
                input.constants.entry(*temporary).or_insert(*constant);
            }
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
