use std::path::Path;

use xic::analyze::analyze;
use xic::analyze::LiveVariables;
use xic::optimize;

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn fold_constants_hir(path: &str) {
    let hir = super::emit_hir(path);

    let hir_stdout = super::interpret_hir(&hir);
    let hir_folded = hir.map(optimize::fold_constants);
    let hir_folded_stdout = super::interpret_hir(&hir_folded);

    pretty_assertions::assert_eq!(hir_stdout, hir_folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn fold_constants_lir(path: &str) {
    let lir = super::emit_lir(path);

    let lir_stdout = super::interpret_lir(&lir);
    let lir_folded = lir.map(optimize::fold_constants);
    let lir_folded_stdout = super::interpret_lir(&lir_folded);

    pretty_assertions::assert_eq!(lir_stdout, lir_folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn eliminate_dead_code_assembly(path: &str) {
    let abstract_assembly = super::tile(path);

    let unoptimized = abstract_assembly.map_ref(xic::api::allocate_trivial);
    let optimized = abstract_assembly
        .map(xic::api::construct_cfg)
        .map(|mut cfg| {
            let live_variables = analyze::<LiveVariables<_>, _>(&cfg);
            optimize::eliminate_dead_code_assembly(&live_variables, &mut cfg);
            cfg
        })
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);

    pretty_assertions::assert_eq!(super::execute(&unoptimized), super::execute(&optimized))
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn propagate_copies_assembly(path: &str) {
    let abstract_assembly = super::tile(path);

    let unoptimized = abstract_assembly.map_ref(xic::api::allocate_trivial);
    let optimized = abstract_assembly
        .map(xic::api::construct_cfg)
        .map_mut(optimize::propagate_copies_assembly)
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);

    pretty_assertions::assert_eq!(super::execute(&unoptimized), super::execute(&optimized))
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn propagate_constants_assembly(path: &str) {
    let abstract_assembly = super::tile(path);

    let unoptimized = abstract_assembly.map_ref(xic::api::allocate_trivial);
    let optimized = abstract_assembly
        .map(xic::api::construct_cfg)
        .map_mut(optimize::propagate_constants_assembly)
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);

    pretty_assertions::assert_eq!(super::execute(&unoptimized), super::execute(&optimized))
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn inline_functions_lir(path: &str) {
    let mut lir = super::reorder(path);

    let unoptimized = super::interpret_lir(&lir);
    optimize::inline_functions_lir(&mut lir);
    let optimized = super::interpret_lir(&lir);

    pretty_assertions::assert_eq!(unoptimized, optimized)
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn eliminate_partial_redundancy_lir(path: &str) {
    let lir = super::emit_lir(path);

    let unoptimized = super::interpret_lir(&lir);

    let lir = lir
        .map(xic::api::construct_cfg)
        .map_mut(optimize::eliminate_partial_redundancy_lir)
        .map(xic::api::destruct_cfg);

    let optimized = super::interpret_lir(&lir);

    pretty_assertions::assert_eq!(unoptimized, optimized)
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn invert_loops_ast(path: &str) {
    let mut program = super::parse(path);

    let context = xic::api::check(Path::new(path).parent().unwrap(), &program).unwrap();
    let unoptimized =
        super::interpret_hir(&xic::api::emit_hir(Path::new(path), &program, &context));

    optimize::invert_loops_ast(&mut program);

    let context = xic::api::check(Path::new(path).parent().unwrap(), &program).unwrap();
    let optimized = super::interpret_hir(&xic::api::emit_hir(Path::new(path), &program, &context));

    pretty_assertions::assert_eq!(unoptimized, optimized)
}
