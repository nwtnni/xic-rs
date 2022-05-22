use std::path::Path;

use xic::analyze::analyze;
use xic::analyze::LiveVariables;
use xic::optimize;

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn fold_constants_hir(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let folded = super::emit_hir(path).map(optimize::fold_constants);
    let folded_stdout = super::interpret_hir(&folded);

    pretty_assertions::assert_eq!(expected_stdout, folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn fold_constants_lir(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let folded = super::emit_lir(path).map(optimize::fold_constants);
    let folded_stdout = super::interpret_lir(&folded);

    pretty_assertions::assert_eq!(expected_stdout, folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn eliminate_dead_code_assembly(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let optimized = super::tile(path)
        .map(xic::api::construct_cfg)
        .map_mut(|cfg| {
            let live_variables = analyze::<LiveVariables<_>, _>(cfg);
            optimize::eliminate_dead_code_assembly(&live_variables, cfg);
        })
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);
    let optimized_stdout = super::execute(&optimized);

    pretty_assertions::assert_eq!(expected_stdout, optimized_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn eliminate_dead_code_lir(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let optimized = super::emit_lir(path)
        .map(xic::api::construct_cfg)
        .map_mut(optimize::eliminate_dead_code_lir)
        .map(xic::api::destruct_cfg);
    let optimized_stdout = super::interpret_lir(&optimized);

    pretty_assertions::assert_eq!(expected_stdout, optimized_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn propagate_copies_assembly(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let optimized = super::tile(path)
        .map(xic::api::construct_cfg)
        .map_mut(optimize::propagate_copies_assembly)
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);
    let optimized_stdout = super::execute(&optimized);

    pretty_assertions::assert_eq!(expected_stdout, optimized_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn propagate_constants_assembly(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let optimized = super::tile(path)
        .map(xic::api::construct_cfg)
        .map_mut(optimize::propagate_constants_assembly)
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);
    let optimized_stdout = super::execute(&optimized);

    pretty_assertions::assert_eq!(expected_stdout, optimized_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn inline_functions_lir(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let mut optimized = super::reorder(path);
    optimize::inline_functions_lir(&mut optimized);
    let optimized_stdout = super::interpret_lir(&optimized);

    pretty_assertions::assert_eq!(expected_stdout, optimized_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn eliminate_partial_redundancy_lir(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let optimized = super::emit_lir(path)
        .map(xic::api::construct_cfg)
        .map_mut(optimize::eliminate_partial_redundancy_lir)
        .map(xic::api::destruct_cfg);
    let optimized_stdout = super::interpret_lir(&optimized);

    pretty_assertions::assert_eq!(expected_stdout, optimized_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn invert_loops_ast(path: &str) {
    let expected_stdout = super::execute_expected(path);

    let mut program = super::parse(path);
    optimize::invert_loops_ast(&mut program);
    let mut context =
        xic::api::check(None, Path::new(path), &program).unwrap();
    let optimized = xic::api::emit_hir(Path::new(path), &program, &mut context);
    let optimized_stdout = super::interpret_hir(&optimized);

    pretty_assertions::assert_eq!(expected_stdout, optimized_stdout);
}
