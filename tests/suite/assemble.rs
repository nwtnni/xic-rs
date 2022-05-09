use std::io;

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn tile(path: &str) {
    let lir = super::reorder(path);

    let mut lir_stdin = io::Cursor::new(Vec::new());
    let mut lir_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_lir(&lir, &mut lir_stdin, &mut lir_stdout).unwrap();
    let lir_stdout = String::from_utf8(lir_stdout.into_inner()).unwrap();

    let assembly = lir
        .map_ref(xic::api::tile)
        .map_ref(xic::api::allocate_trivial);
    let assembly_stdout = super::execute(&assembly);
    pretty_assertions::assert_eq!(lir_stdout, assembly_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn reorder(path: &str) {
    let abstract_assembly = super::tile(path);

    let control = abstract_assembly.map_ref(xic::api::allocate_trivial);
    let control_stdout = super::execute(&control);

    let cfg = abstract_assembly.map(xic::api::construct_cfg);

    let reordered = cfg
        .clone()
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);
    let reordered_stdout = super::execute(&reordered);

    pretty_assertions::assert_eq!(control_stdout, reordered_stdout);

    let cleaned = cfg
        .map_mut(xic::api::clean_cfg)
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);
    let cleaned_stdout = super::execute(&cleaned);

    pretty_assertions::assert_eq!(reordered_stdout, cleaned_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn allocate(path: &str) {
    let lir = super::tile(path);

    let trivial = lir.map_ref(xic::api::allocate_trivial);
    let linear = lir
        .map(xic::api::construct_cfg)
        .map(xic::api::allocate_linear);

    pretty_assertions::assert_eq!(super::execute(&trivial), super::execute(&linear));
}
