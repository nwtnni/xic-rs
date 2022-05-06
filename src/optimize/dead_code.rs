use crate::analyze::analyze;
use crate::analyze::Analysis as _;
use crate::analyze::LiveVariables;
use crate::cfg;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::util::Or;

pub fn eliminate(cfg: &Cfg<asm::Function<Temporary>>) -> asm::Function<Temporary> {
    let (analysis, mut solution) = analyze::<LiveVariables<_>, _>(cfg);
    let mut cfg = cfg.clone();

    for (label, statements) in cfg.blocks_mut() {
        let output = solution.inputs.get_mut(label).unwrap();

        for statement in statements.iter_mut().rev() {
            let destination = match statement {
                asm::Statement::Binary(asm::Binary::Cmp, _) => None,
                asm::Statement::Binary(
                    asm::Binary::Add
                    | asm::Binary::Sub
                    | asm::Binary::Shl
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
                asm::Statement::Unary(asm::Unary::Mul | asm::Unary::Div, _) => {
                    Some(Temporary::Register(Register::Rax))
                }
                asm::Statement::Unary(asm::Unary::Hul | asm::Unary::Mod, _) => {
                    Some(Temporary::Register(Register::Rdx))
                }
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

            analysis.transfer(statement, output);

            if !live {
                *statement = asm::Statement::Nullary(asm::Nullary::Nop);
            }
        }
    }

    let mut function = cfg::destruct_cfg(&cfg);
    function
        .statements
        .retain(|statement| !matches!(statement, asm::Statement::Nullary(asm::Nullary::Nop)));
    function
}
