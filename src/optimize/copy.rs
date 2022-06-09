use crate::analyze::analyze;
use crate::analyze::Analysis;
use crate::analyze::CopyPropagation;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::asm::Statement;
use crate::data::operand;
use crate::data::operand::Temporary;
use crate::util;
use crate::Map;

pub fn propagate_assembly(cfg: &mut Cfg<asm::Function<Temporary>>) {
    log::info!(
        "[{}] Propagating copies in {}...",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );
    util::time!(
        "[{}] Done propagating copies in {}",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );

    let mut solution = analyze::<CopyPropagation, _>(cfg);

    for (label, statements) in cfg.blocks_mut() {
        let mut output = solution.inputs.remove(label).unwrap();

        for statement in statements {
            use asm::Binary::*;
            use asm::Nullary::*;
            use asm::Unary::*;

            let save = statement.clone();

            match statement {
                // `cmp` is a bit special because it doesn't modify its destination.
                // So it should be fine to rewrite something like this:
                //
                // ```
                // mov t0, a     mov t0, a
                // mov t1, b  -> mov t1, b
                // cmp t0, t1    cmp a, b
                // ```
                asm::Statement::Binary(
                    Cmp,
                    operand::Binary::RR {
                        destination,
                        source,
                    },
                ) => {
                    *destination = traverse(&output, destination);
                    *source = traverse(&output, source);
                }
                asm::Statement::Binary(Cmp, operand::Binary::RM { destination, .. }) => {
                    *destination = traverse(&output, destination);
                }
                asm::Statement::Binary(Cmp, operand::Binary::RI { destination, .. }) => {
                    *destination = traverse(&output, destination);
                }

                Statement::Binary(
                    Mov | Lea | Add | Sub | Shl | Mul | And | Or | Xor,
                    operand::Binary::RR { source, .. },
                ) => {
                    *source = traverse(&output, source);
                }
                Statement::Binary(
                    Cmp | Mov | Lea | Add | Sub | Shl | And | Or | Xor,
                    operand::Binary::MR { source, .. },
                ) => {
                    *source = traverse(&output, source);
                }

                Statement::Unary(Hul | Div | Mod | Call { .. }, operand::Unary::R(source)) => {
                    *source = traverse(&output, source);
                }

                Statement::Binary(Cmp | Mov | Lea | Add | Sub | Shl | Mul | And | Or | Xor, _)
                | Statement::Unary(Neg | Hul | Div | Mod | Call { .. }, _)
                | Statement::Nullary(Nop | Cqo | Ret(_))
                | Statement::Label(_)
                | Statement::Jmp(_)
                | Statement::Jcc(_, _) => (),
            };

            <CopyPropagation as Analysis<asm::Function<Temporary>>>::transfer(
                &solution.analysis,
                &save,
                &mut output,
            );
        }
    }
}

fn traverse(output: &Map<Temporary, Temporary>, temporary: &Temporary) -> Temporary {
    match output.get(temporary) {
        None => *temporary,
        Some(temporary) => traverse(output, temporary),
    }
}
