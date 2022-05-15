use crate::analyze::analyze;
use crate::analyze::Analysis as _;
use crate::analyze::LiveVariables;
use crate::analyze::Solution;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::util::Or;

pub fn eliminate_assembly(
    live_variables: &Solution<LiveVariables<asm::Function<Temporary>>, asm::Function<Temporary>>,
    cfg: &mut Cfg<asm::Function<Temporary>>,
) {
    for (label, statements) in cfg.blocks_mut() {
        let mut output = live_variables.inputs[label].clone();

        for statement in statements.iter_mut().rev() {
            let destination = match statement {
                asm::Statement::Binary(asm::Binary::Cmp, _) => None,
                asm::Statement::Binary(
                    asm::Binary::Add
                    | asm::Binary::Sub
                    | asm::Binary::Shl
                    | asm::Binary::Mul
                    | asm::Binary::And
                    | asm::Binary::Or
                    | asm::Binary::Xor
                    | asm::Binary::Mov
                    | asm::Binary::Lea,
                    operands,
                ) => match operands.destination() {
                    Or::L(Temporary::Register(register)) if register.is_callee_saved() => None,
                    Or::L(temporary) => Some(temporary),
                    Or::R(_) => None,
                },
                asm::Statement::Unary(asm::Unary::Hul, _) => {
                    Some(Temporary::Register(Register::Rdx))
                }
                asm::Statement::Unary(asm::Unary::Div | asm::Unary::Mod, _) => None,
                asm::Statement::Unary(asm::Unary::Neg | asm::Unary::Call { .. }, operand) => {
                    match operand {
                        operand::Unary::M(_) | operand::Unary::I(_) => None,
                        operand::Unary::R(temporary) => Some(*temporary),
                    }
                }
                asm::Statement::Nullary(asm::Nullary::Cqo) => {
                    Some(Temporary::Register(Register::Rdx))
                }
                asm::Statement::Nullary(asm::Nullary::Nop | asm::Nullary::Ret(_))
                | asm::Statement::Label(_)
                | asm::Statement::Jmp(_)
                | asm::Statement::Jcc(_, _) => None,
            };

            let live = destination.map_or(true, |destination| output.contains(&destination));

            live_variables.analysis.transfer(statement, &mut output);

            if !live {
                *statement = asm::Statement::Nullary(asm::Nullary::Nop);
            }
        }
    }
}

pub fn eliminate_lir<T: lir::Target>(cfg: &mut Cfg<lir::Function<T>>) {
    let mut live_variables = analyze::<LiveVariables<_>, _>(cfg);
    let mut buffer = Vec::new();

    for (label, statements) in cfg.blocks_mut() {
        let mut output = live_variables.inputs.remove(label).unwrap();

        buffer.append(statements);

        for statement in buffer.drain(..).rev() {
            let destination = match &statement {
                lir::Statement::Jump(_)
                | lir::Statement::CJump { .. }
                | lir::Statement::Call(_, _, _)
                | lir::Statement::Label(_)
                | lir::Statement::Return(_) => None,
                lir::Statement::Move {
                    destination: lir::Expression::Memory(_),
                    source: _,
                } => None,
                lir::Statement::Move {
                    destination: lir::Expression::Temporary(temporary),
                    source: _,
                } => Some(temporary),
                lir::Statement::Move { .. } => unreachable!(),
            };

            let live = destination.map_or(true, |destination| output.contains(destination));

            live_variables.analysis.transfer(&statement, &mut output);

            if live {
                statements.push(statement);
            }
        }

        statements.reverse();
    }
}
