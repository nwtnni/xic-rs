use crate::abi;
use crate::analyze::analyze_default;
use crate::analyze::Analysis as _;
use crate::analyze::CallGraph;
use crate::analyze::LiveVariables;
use crate::analyze::Solution;
use crate::cfg::Cfg;
use crate::data::asm;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::util;
use crate::util::Or;
use crate::Set;

pub fn eliminate_functions<T: lir::Target>(lir: &mut ir::Unit<Cfg<lir::Function<T>>>) {
    let call_graph = CallGraph::new(lir);
    let mut eliminated = 0;

    // A function is reachable if it is reachable from `init` or a globally visible function.
    // We enforce that `main` is always globally visible.
    let reachable = call_graph
        .postorder(&symbol::intern_static(abi::XI_INIT_CLASSES))
        .chain(call_graph.postorder(&symbol::intern_static(abi::XI_INIT_GLOBALS)))
        .chain(
            lir.functions
                .values()
                .filter(|cfg| match cfg.metadata() {
                    (_, _, ir::Linkage::Global) => true,
                    (_, _, ir::Linkage::Local | ir::Linkage::LinkOnceOdr) => false,
                })
                .flat_map(|cfg| call_graph.postorder(cfg.name())),
        )
        .collect::<Set<_>>();

    lir.functions.retain(|_, function| {
        let (_, _, linkage) = function.metadata();
        match linkage {
            ir::Linkage::Global => return true,
            ir::Linkage::Local | ir::Linkage::LinkOnceOdr => (),
        }

        if reachable.contains(function.name()) {
            return true;
        }

        log::trace!("Eliminated dead function: {}", function.name());
        eliminated += 1;
        false
    });

    log::debug!("Eliminated {} dead functions!", eliminated);
}

pub fn eliminate_lir<T: lir::Target>(cfg: &mut Cfg<lir::Function<T>>) {
    log::info!(
        "[{}] Eliminating dead code in {}...",
        std::any::type_name::<Cfg<lir::Function<T>>>(),
        cfg.name()
    );
    util::time!(
        "[{}] Done eliminating dead code in {}",
        std::any::type_name::<Cfg<lir::Function<T>>>(),
        cfg.name()
    );

    let mut live_variables = analyze_default::<LiveVariables<_>, _>(cfg);
    let mut eliminated = 0;
    let mut buffer = Vec::new();

    for (label, statements) in cfg.blocks_mut() {
        let mut output = live_variables.inputs.remove(label).unwrap();

        buffer.append(statements);

        // Reverse order is necessary for backward analysis
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
            } else {
                log::trace!("Eliminated dead LIR statement: {}", statement);
                eliminated += 1;
            }
        }

        statements.reverse();
    }

    log::debug!("Eliminated {} dead LIR statements!", eliminated);
}

pub fn eliminate_assembly(
    live_variables: &Solution<LiveVariables<asm::Function<Temporary>>, asm::Function<Temporary>>,
    cfg: &mut Cfg<asm::Function<Temporary>>,
) {
    log::info!(
        "[{}] Eliminating dead code in {}...",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );
    util::time!(
        "[{}] Done eliminating dead code in {}",
        std::any::type_name::<Cfg<asm::Function<Temporary>>>(),
        cfg.name()
    );

    let mut eliminated = 0;

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
                // Preserve statements that may have side effects
                asm::Statement::Unary(
                    asm::Unary::Div
                    | asm::Unary::Mod
                    | asm::Unary::Call { .. }
                    | asm::Unary::Push
                    | asm::Unary::Pop,
                    _,
                ) => None,
                asm::Statement::Unary(asm::Unary::Neg, operand) => match operand {
                    operand::Unary::M(_) | operand::Unary::I(_) => None,
                    operand::Unary::R(temporary) => Some(*temporary),
                },
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
                log::trace!("Eliminated dead assembly statement: {}", statement);
                eliminated += 1;
                *statement = asm::Statement::Nullary(asm::Nullary::Nop);
            }
        }
    }

    log::debug!("Eliminated {} dead assembly statements!", eliminated);
}
