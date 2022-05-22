#[path = "suite/lex.rs"]
mod lex;

#[path = "suite/parse.rs"]
mod parse;

#[path = "suite/check.rs"]
mod check;

#[path = "suite/emit.rs"]
mod emit;

#[path = "suite/assemble.rs"]
mod assemble;

#[path = "suite/analyze.rs"]
mod analyze;

#[path = "suite/optimize.rs"]
mod optimize;

#[path = "suite/end_to_end.rs"]
mod end_to_end;

use std::fmt::Display;
use std::fs;
use std::io::Cursor;
use std::io::Write as _;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use tempfile::NamedTempFile;

use xic::data::asm;
use xic::data::ast;
use xic::data::hir;
use xic::data::lir;
use xic::data::operand::Label;
use xic::data::operand::Temporary;
use xic::data::token::Tokens;

pub fn lex(path: &str) -> Tokens {
    xic::api::lex(Path::new(path)).unwrap()
}

pub fn parse(path: &str) -> ast::Program {
    xic::api::parse(lex(path)).unwrap()
}

pub fn emit_hir(path: &str) -> hir::Unit {
    let program = parse(path);
    let mut context = xic::api::check(None, Path::new(path), &program).unwrap();
    xic::api::emit_hir(Path::new(path), &program, &mut context)
}

pub fn emit_lir(path: &str) -> lir::Unit<Label> {
    emit_hir(path).map_ref(xic::api::emit_lir)
}

pub fn reorder(path: &str) -> lir::Unit<lir::Fallthrough> {
    emit_lir(path)
        .map(xic::api::construct_cfg)
        .map(xic::api::destruct_cfg)
}

pub fn tile(path: &str) -> asm::Unit<Temporary> {
    reorder(path).map_ref(xic::api::tile)
}

pub fn interpret_hir(hir: &hir::Unit) -> String {
    let mut stdin = Cursor::new(Vec::new());
    let mut stdout = Cursor::new(Vec::new());
    xic::api::interpret_hir(hir, &mut stdin, &mut stdout).unwrap();
    String::from_utf8(stdout.into_inner()).unwrap()
}

pub fn interpret_lir<T: lir::Target>(lir: &lir::Unit<T>) -> String {
    let mut stdin = Cursor::new(Vec::new());
    let mut stdout = Cursor::new(Vec::new());
    xic::api::interpret_lir(lir, &mut stdin, &mut stdout).unwrap();
    String::from_utf8(stdout.into_inner()).unwrap()
}

pub fn execute_expected(path: &str) -> String {
    let path = format!(
        "{}/tests/suite/snapshots/suite__emit__tests__execute__{}.snap",
        env!("CARGO_MANIFEST_DIR"),
        Path::new(path).file_name().unwrap().to_str().unwrap(),
    );

    let snap = fs::read_to_string(&path).unwrap();
    let (_, stdout) = snap
        .trim_start_matches("---\n")
        .split_once("---\n")
        .unwrap();

    String::from(stdout.strip_suffix('\n').unwrap())
}

pub fn execute<T: Display>(assembly: T) -> String {
    let path = NamedTempFile::new().unwrap().into_temp_path();

    let mut cc = Command::new("cc")
        .stdin(Stdio::piped())
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

    write!(&mut stdin, "{}", assembly).unwrap();
    stdin.flush().unwrap();
    drop(stdin);

    if !cc.wait().unwrap().success() {
        panic!("Assembly compilation failed");
    }

    Command::new(&path)
        .output()
        .map(|output| output.stdout)
        .map(String::from_utf8)
        .unwrap()
        .unwrap()
}

pub fn graph<T: Display>(dot: T) -> String {
    let mut graph = Command::new("graph-easy")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("-")
        .spawn()
        .unwrap();

    write!(&mut graph.stdin.as_mut().unwrap(), "{}", dot).unwrap();

    match graph.wait_with_output() {
        Ok(output) if !output.status.success() => panic!("Failed to process .dot output"),
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(error) => panic!("Failed to execute `graph-easy`: {}", error),
    }
}
