use std::io;
use std::io::Write as _;
use std::path::Path;
use std::process;

use xic::data::lir;

fn compile(path: &str) -> lir::Unit<lir::Fallthrough> {
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    let program = xic::api::parse(tokens).unwrap();
    let context = xic::api::check(Path::new(path).parent().unwrap(), &program).unwrap();
    let hir = xic::api::emit_hir(Path::new(path), &program, &context);
    let lir = xic::api::emit_lir(&hir);
    let cfg = xic::api::construct_control_flow(&lir);
    xic::api::destruct_control_flow(&cfg)
}

fn execute(lir: &lir::Unit<lir::Fallthrough>) -> String {
    let abstract_assembly = xic::api::tile(lir);
    let assembly = xic::api::allocate(&abstract_assembly);

    let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();

    let mut cc = process::Command::new("cc")
        .stdin(process::Stdio::piped())
        .arg("-L")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime"))
        .arg("-lxi")
        .arg("-lpthread")
        .arg("-xassembler")
        .arg("-o")
        .arg(&path)
        .arg("-")
        .spawn()
        .unwrap();

    let mut stdin = cc.stdin.take().unwrap();

    write!(&mut stdin, "{}", assembly.intel()).unwrap();
    stdin.flush().unwrap();
    drop(stdin);

    if !cc.wait().unwrap().success() {
        panic!("Assembly compilation failed");
    }

    process::Command::new(&path)
        .output()
        .map(|output| output.stdout)
        .map(String::from_utf8)
        .unwrap()
        .unwrap()
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn tile(path: &str) {
    let lir = compile(path);

    let mut lir_stdin = io::Cursor::new(Vec::new());
    let mut lir_stdout = io::Cursor::new(Vec::new());
    xic::api::interpret_lir(&lir, &mut lir_stdin, &mut lir_stdout).unwrap();

    let lir_stdout = String::from_utf8(lir_stdout.into_inner()).unwrap();
    let assembly_stdout = execute(&lir);

    pretty_assertions::assert_eq!(lir_stdout, assembly_stdout);
}
