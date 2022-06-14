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

use anyhow::anyhow;
use anyhow::Context as _;
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
    xic::api::parse(Path::new(path), lex(path)).unwrap()
}

pub fn emit_hir(path: &str) -> hir::Unit {
    let mut program = parse(path);
    let mut context = xic::api::check(None, Path::new(path), &mut program).unwrap();
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

pub fn execute<T: Display>(object: T) -> String {
    let path = NamedTempFile::new().unwrap().into_temp_path();

    let mut cc = Command::new("cc");
    cc.arg("-xassembler")
        .arg("-")
        .arg("-L")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime"))
        .arg("-lxi")
        .arg("-lpthread")
        .arg("-o")
        .arg(&path);

    let _ = stdout(cc, Some(object))
        .context("Assembling with `cc`")
        .unwrap();

    stdout(Command::new(&path), None::<String>)
        .context("Running assembled binary")
        .unwrap()
}

pub fn execute_all<I, T>(objects: I) -> String
where
    I: IntoIterator<Item = T>,
    T: Display,
{
    let paths = objects
        .into_iter()
        .map(|object| {
            let mut file = NamedTempFile::new().unwrap();
            write!(&mut file, "{}", object).unwrap();
            file.flush().unwrap();
            file.into_temp_path()
        })
        .collect::<Vec<_>>();

    let path = NamedTempFile::new().unwrap().into_temp_path();

    let mut cc = Command::new("cc");
    cc.arg("-xassembler")
        .arg("-L")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime"))
        // See: https://stackoverflow.com/questions/5651869/what-are-the-start-group-and-end-group-command-line-options
        .arg("-Wl,--start-group")
        .args(&paths)
        .arg("-lxi")
        .arg("-Wl,--end-group")
        .arg("-lpthread")
        .arg("-o")
        .arg(&path);

    let _ = stdout(cc, None::<String>)
        .context("Assembling with `cc`")
        .unwrap();

    stdout(Command::new(&path), None::<String>)
        .context("Running assembled binary")
        .unwrap()
}

pub fn graph<T: Display>(dot: T) -> String {
    let mut graph = Command::new("graph-easy");
    graph.arg("-");
    stdout(graph, Some(dot))
        .context("Running `graph-easy` on `.dot` output")
        .unwrap()
}

pub fn stdout<T: Display>(mut command: Command, stdin: Option<T>) -> anyhow::Result<String> {
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Spawning process")?;

    if let Some(stdin) = stdin {
        write!(child.stdin.as_mut().unwrap(), "{}", stdin).context("Writing to `stdin`")?;
        child
            .stdin
            .as_mut()
            .unwrap()
            .flush()
            .context("Flushing `stdin`")?;
    }

    let output = child
        .wait_with_output()
        .context("Waiting for process output")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(anyhow!(
            "Command failed with exit code `{:?}` and stderr: `{}`",
            output.status.code(),
            stderr,
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
