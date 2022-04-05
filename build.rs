fn main() {
    lalrpop::Configuration::new()
        .emit_whitespace(false)
        .process_file("src/parse/parser.lalrpop")
        .unwrap();
}
