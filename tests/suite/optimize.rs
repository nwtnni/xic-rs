use xic::api::analyze::analyze;
use xic::api::analyze::LiveVariables;

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn constant_fold_hir(path: &str) {
    let hir = super::emit_hir(path);

    let hir_stdout = super::interpret_hir(&hir);
    let hir_folded = hir.map(xic::api::optimize::constant_fold);
    let hir_folded_stdout = super::interpret_hir(&hir_folded);

    pretty_assertions::assert_eq!(hir_stdout, hir_folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn constant_fold_lir(path: &str) {
    let lir = super::emit_lir(path);

    let lir_stdout = super::interpret_lir(&lir);
    let lir_folded = lir.map(xic::api::optimize::constant_fold);
    let lir_folded_stdout = super::interpret_lir(&lir_folded);

    pretty_assertions::assert_eq!(lir_stdout, lir_folded_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn eliminate_dead_code(path: &str) {
    let abstract_assembly = super::tile(path);

    let unoptimized = abstract_assembly.map_ref(xic::api::allocate_trivial);
    let optimized = abstract_assembly
        .map(xic::api::construct_cfg)
        .map(|mut cfg| {
            let live_variables = analyze::<LiveVariables<_>, _>(&cfg);
            xic::api::optimize::eliminate_dead_code(&live_variables, &mut cfg);
            cfg
        })
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);

    pretty_assertions::assert_eq!(super::execute(&unoptimized), super::execute(&optimized))
}
