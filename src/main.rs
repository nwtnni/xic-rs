use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use structopt::StructOpt;

use xic::data::sexp::Serialize as _;

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

    /// Interpret emitted IR
    #[structopt(short = "r", long = "irrun")]
    interpret_ir: bool,

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
    pretty_env_logger::init_timed();

    let command = Command::from_args();

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

        let mut hir = xic::api::emit_hir(&path, &program, &context?);

        if !command.optimize_disable {
            hir = xic::api::fold_hir(hir);
        }

        if command.debug_ir {
            write!(
                debug(&command.directory_output, &path, "hir")?,
                "{}",
                hir.sexp(),
            )?;
        }

        let mut lir = xic::api::emit_lir(&hir);

        if !command.optimize_disable {
            lir = xic::api::fold_lir(lir);
        }

        if command.debug_ir {
            write!(
                debug(&command.directory_output, &path, "lir")?,
                "{}",
                lir.sexp(),
            )?;
        }

        if command.interpret_ir {
            xic::api::interpret_lir(&lir, io::BufReader::new(io::stdin()), io::stdout())?;
        }
    }

    Ok(())
}

fn debug(directory: &Path, path: &Path, extension: &str) -> io::Result<io::BufWriter<fs::File>> {
    fs::File::create(directory.join(path).with_extension(extension)).map(io::BufWriter::new)
}
