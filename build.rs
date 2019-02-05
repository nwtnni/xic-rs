fn main() {
    lalrpop::Configuration::new()
        .generate_in_source_tree()
        .emit_whitespace(false)
        .force_build(true)
        .process()
        .unwrap();
}
