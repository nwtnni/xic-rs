fn main() {
    lalrpop::Configuration::new()
        .emit_whitespace(false)
        .generate_in_source_tree()
        .process_file("src/parse/parser.lalrpop")
        .unwrap();
}
