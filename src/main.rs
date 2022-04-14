use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use structopt::StructOpt;

use xic::emit;
use xic::util::sexp::Serialize as _;
use xic::util::Tap as _;

#[derive(Debug, StructOpt)]
#[structopt(name = "xic", about = "Compiler for the Xi programming language.")]
struct Command {
    /// Generate output from lexical analysis
    #[structopt(short = "l", long = "lex")]
    debug_lex: bool,

    /// Generate output from syntactic analysis
    #[structopt(short = "p", long = "parse")]
    debug_parse: bool,

    /// Generate output from semantic analysis
    #[structopt(short = "t", long = "typecheck")]
    debug_check: bool,

    /// Generate output from emitted IR
    #[structopt(short = "g", long = "irgen")]
    debug_ir: bool,

    /// Emulate emitted IR
    #[structopt(short = "r", long = "irrun")]
    run_ir: bool,

    /// Disable optimizations
    #[structopt(short = "O")]
    optimize_disable: bool,

    /// Specify where to place generated diagnostic files
    #[structopt(short = "D", parse(from_os_str), default_value = ".")]
    directory_output: PathBuf,

    /// Specify where to search for source files
    #[structopt(long = "sourcepath", parse(from_os_str), default_value = ".")]
    directory_source: PathBuf,

    /// Specify where to search for library files
    #[structopt(long = "libpath", parse(from_os_str))]
    directory_library: Option<PathBuf>,

    /// Source files to compile, relative to `source_dir`
    #[structopt(parse(from_os_str))]
    input: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let command = Command::from_args();

    let emitter = emit::Driver::new(
        &command.directory_output,
        command.debug_ir,
        !command.optimize_disable,
        command.run_ir,
    );

    for path in &command.input {
        let path = command.directory_source.join(path);

        let tokens = xic::api::lex(&path)?;

        if command.debug_lex {
            write!(
                debug(&command.directory_output, &path, "lexed")?,
                "{}",
                tokens
            )?;
        }

        let program = xic::api::parse(tokens)?;

        if command.debug_parse {
            write!(
                debug(&command.directory_output, &path, "parsed")?,
                "{}",
                program.sexp(),
            )?;
        }

        let context = xic::api::check(
            command
                .directory_library
                .as_deref()
                .or_else(|| path.parent())
                .unwrap(),
            &program,
        );

        if command.debug_check {
            let mut file = debug(&command.directory_output, &path, "typed")?;
            match &context {
                Ok(_) => write!(file, "Valid Xi Program")?,
                Err(error) => write!(file, "{}", error)?,
            }
        }

        let hir = emitter.emit_hir(&path, &program, &context?)?;
        let _lir = emitter.emit_lir(&path, &hir)?;
    }

    Ok(())
}

fn debug(directory: &Path, path: &Path, extension: &str) -> io::Result<io::BufWriter<fs::File>> {
    directory
        .join(path)
        .with_extension(extension)
        .tap(fs::File::create)
        .map(io::BufWriter::new)
}
