use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::str;

use anyhow::anyhow;
use structopt::StructOpt;

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

    /// Generate DOT file from IR CFG
    #[structopt(short = "c", long = "optcfg")]
    debug_cfg: bool,

    /// Generate output from abstract assembly
    #[structopt(short = "a", long = "abstract-assembly")]
    debug_abstract_assembly: bool,

    /// Interpret emitted IR
    #[structopt(short = "r", long = "irrun")]
    interpret_ir: bool,

    /// Enable optimizations
    #[structopt(short = "O")]
    optimize: Option<Vec<Optimization>>,

    /// Specify where to place generated diagnostic files
    #[structopt(short = "D", parse(from_os_str), default_value = ".")]
    directory_debug: PathBuf,

    /// Specify where to place generated assembly files
    #[structopt(short = "d", parse(from_os_str), default_value = ".")]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Optimization {
    ConstantFold,
    DeadCodeElimination,
    RegisterAllocation,
}

impl str::FromStr for Optimization {
    type Err = anyhow::Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "cf" => Ok(Optimization::ConstantFold),
            "dec" => Ok(Optimization::DeadCodeElimination),
            "reg" => Ok(Optimization::RegisterAllocation),
            _ => Err(anyhow!(
                "Unknown optimization {}, expected one of [cf, reg]",
                string
            )),
        }
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init_timed();

    let command = Command::from_args();

    for path in &command.input {
        let path = command.directory_source.join(path);

        let tokens = xic::api::lex(&path)?;

        if command.debug_lex {
            write!(
                debug(&command.directory_debug, &path, "lexed")?,
                "{}",
                tokens
            )?;
        }

        let program = xic::api::parse(tokens)?;

        if command.debug_parse {
            write!(
                debug(&command.directory_debug, &path, "parsed")?,
                "{}",
                program,
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
            let mut file = debug(&command.directory_debug, &path, "typed")?;
            match &context {
                Ok(_) => write!(file, "Valid Xi Program")?,
                Err(error) => write!(file, "{}", error)?,
            }
        }

        let mut hir = xic::api::emit_hir(&path, &program, &context?);

        if command.optimize.as_ref().map_or(true, |optimizations| {
            optimizations.contains(&Optimization::ConstantFold)
        }) {
            hir = xic::api::constant_fold_hir(hir);
        }

        if command.debug_ir {
            write!(debug(&command.directory_debug, &path, "hir")?, "{}", hir,)?;
        }

        let mut lir = hir.map(xic::api::emit_lir);

        if command.optimize.as_ref().map_or(true, |optimizations| {
            optimizations.contains(&Optimization::ConstantFold)
        }) {
            lir = xic::api::constant_fold_lir(lir);
        }

        let cfg = lir.map(xic::api::construct_cfg);

        if command.debug_cfg {
            write!(debug(&command.directory_debug, &path, "dot")?, "{}", cfg)?;
        }

        let mut lir = cfg.map(xic::api::destruct_cfg);

        if command.optimize.as_ref().map_or(true, |optimizations| {
            optimizations.contains(&Optimization::ConstantFold)
        }) {
            lir = xic::api::constant_fold_lir(lir);
        }

        if command.debug_ir {
            write!(debug(&command.directory_debug, &path, "lir")?, "{}", lir,)?;
        }

        if command.interpret_ir {
            xic::api::interpret_lir(&lir, io::BufReader::new(io::stdin()), io::stdout())?;
        }

        let abstract_assembly = lir.map(xic::api::tile);

        if command.debug_abstract_assembly {
            write!(
                debug(&command.directory_debug, &path, "tiled")?,
                "{}",
                abstract_assembly.intel(),
            )?;
        }

        let assembly = if command.optimize.as_ref().map_or(true, |optimizations| {
            optimizations.contains(&Optimization::RegisterAllocation)
        }) {
            abstract_assembly.map(xic::api::allocate_linear)
        } else if command.optimize.as_ref().map_or(true, |optimizations| {
            optimizations.contains(&Optimization::DeadCodeElimination)
        }) {
            abstract_assembly
                .map(xic::api::construct_cfg)
                .map(xic::api::eliminate_dead_code)
                .map(xic::api::allocate_trivial)
        } else {
            abstract_assembly.map(xic::api::allocate_trivial)
        };

        write!(
            debug(&command.directory_output, &path, "S")?,
            "{}",
            assembly.intel(),
        )?;
    }

    Ok(())
}

fn debug(directory: &Path, path: &Path, extension: &str) -> io::Result<io::BufWriter<fs::File>> {
    fs::File::create(directory.join(path).with_extension(extension)).map(io::BufWriter::new)
}
