use std::io;
use std::io::Write as _;
use std::path::Path;
use std::process;

use xic::data::asm;
use xic::data::lir;
use xic::data::operand::Register;

fn compile(path: &str) -> lir::Unit<lir::Fallthrough> {
    let tokens = xic::api::lex(Path::new(path)).unwrap();
    let program = xic::api::parse(tokens).unwrap();
    let context = xic::api::check(Path::new(path).parent().unwrap(), &program).unwrap();
    xic::api::emit_hir(Path::new(path), &program, &context)
        .map_ref(xic::api::emit_lir)
        .map(xic::api::construct_cfg)
        .map(xic::api::destruct_cfg)
}

fn execute(assembly: &asm::Unit<Register>) -> String {
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

    let assembly = lir
        .map_ref(xic::api::tile)
        .map_ref(xic::api::allocate_trivial);
    let assembly_stdout = execute(&assembly);
    pretty_assertions::assert_eq!(lir_stdout, assembly_stdout);
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn reorder(path: &str) {
    let abstract_assembly = compile(path).map_ref(xic::api::tile);

    let before = abstract_assembly.map_ref(xic::api::allocate_trivial);
    let after = abstract_assembly
        .map(xic::api::construct_cfg)
        .map(xic::api::destruct_cfg)
        .map_ref(xic::api::allocate_trivial);

    pretty_assertions::assert_eq!(execute(&before), execute(&after));
}

#[test_generator::test_resources("tests/execute/*.xi")]
pub fn allocate(path: &str) {
    let lir = compile(path).map_ref(xic::api::tile);

    let trivial = lir.map_ref(xic::api::allocate_trivial);
    let linear = lir
        .map(xic::api::construct_cfg)
        .map(xic::api::allocate_linear);

    pretty_assertions::assert_eq!(execute(&trivial), execute(&linear));
}
