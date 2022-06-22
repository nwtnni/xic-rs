use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::str;

use anyhow::anyhow;
use anyhow::Context as _;
use clap::Parser;
use clap::ValueHint;
use xic::analyze;
use xic::api;
use xic::optimize;

#[derive(Debug, Parser)]
#[clap(name = "xic", about = "Compiler for the Xi programming language.")]
struct Command {
    /// Directory to search for source files
    #[clap(
        long = "sourcepath",
        default_value = ".",
        value_hint = ValueHint::DirPath,
        display_order = 0,
    )]
    directory_source: PathBuf,

    /// Directory to search for library files
    ///
    /// Defaults to the same directory as the source file using the library.
    #[clap(
        long = "libpath",
        value_hint = ValueHint::DirPath,
        display_order = 1,
    )]
    directory_library: Option<PathBuf>,

    /// Directory to place generated diagnostic files
    ///
    /// If provided with `-`, output to `stdout`.
    #[clap(
        short = 'D',
        default_value = ".",
        value_hint = ValueHint::DirPath,
        display_order = 2,
    )]
    directory_debug: PathBuf,

    /// Directory to place final assembly files
    ///
    /// If provided with `-`, output to `stdout`.
    #[clap(
        short = 'd',
        default_value = ".",
        value_hint = ValueHint::DirPath,
        display_order = 3,
    )]
    directory_output: PathBuf,

    /// Enable optimizations
    ///
    /// Takes a comma-separated list of optimzations to enable.
    #[clap(
        short = 'O',
        long = "O",
        id = "optimize-enable",
        conflicts_with = "optimize-disable",
        use_value_delimiter = true,
        require_value_delimiter = true,
        multiple_occurrences = false,
        min_values = 0,
        value_name = "OPTIMIZATION",
        possible_values = OPTIMIZATIONS,
        display_order = 4,
    )]
    optimize_enable: Option<Vec<Opt>>,

    /// Disable optimizations
    ///
    /// Takes a comma-separated list of optimzations to disable.
    #[clap(
        short = 'o',
        long = "O-no",
        id = "optimize-disable",
        conflicts_with = "optimize-enable",
        use_value_delimiter = true,
        require_value_delimiter = true,
        multiple_occurrences = false,
        min_values = 1,
        value_name = "OPTIMIZATION",
        possible_values = OPTIMIZATIONS,
        display_order = 5,
    )]
    optimize_disable: Option<Vec<Opt>>,

    /// Generate output from lexical analysis
    #[clap(short = 'l', long = "lex", display_order = 6)]
    debug_lex: bool,

    /// Generate output from syntactic analysis
    #[clap(short = 'p', long = "parse", display_order = 7)]
    debug_parse: bool,

    /// Generate output from semantic analysis
    #[clap(short = 't', long = "typecheck", display_order = 8)]
    debug_check: bool,

    /// Generate output from emitted HIR, before lowering, reordering, and CFG optimization
    #[clap(short = 'h', long = "hirgen", display_order = 9)]
    debug_hir: bool,

    /// Generate IR CFG output in DOT format after optimization phases
    ///
    /// Takes a comma-separated list of optimization names.
    #[clap(
        long = "optir",
        use_value_delimiter = true,
        require_value_delimiter = true,
        value_name = "OPTIMIZATION",
        possible_values = [
            DebugOpt::Initial.to_static_str(),
            Opt::CleanCfg.to_static_str(),
            Opt::Inline.to_static_str(),
            Opt::CopyPropagation.to_static_str(),
            Opt::ConditionalConstantPropagation.to_static_str(),
            Opt::DeadCodeElimination.to_static_str(),
            Opt::PartialRedundancyElimination.to_static_str(),
            DebugOpt::Final.to_static_str(),
        ],
        display_order = 10,
    )]
    debug_optimize_lir: Vec<DebugOpt>,

    /// Generate output from emitted LIR, after lowering, reordering, and CFG optimization
    #[clap(short = 'g', long = "irgen", alias = "lirgen", display_order = 11)]
    debug_lir: bool,

    /// Interpret emitted IR
    #[clap(short = 'r', long = "irrun", display_order = 12)]
    interpret_ir: bool,

    /// Generate output from abstract assembly, before CFG optimization
    #[clap(short = 'a', long = "tile", display_order = 13)]
    debug_assembly: bool,

    /// Generate abstract assembly CFG in DOT format after optimization phases
    ///
    /// Takes a comma-separated list of optimization names.
    #[clap(
        long = "optcfg",
        use_value_delimiter = true,
        require_value_delimiter = true,
        value_name = "OPTIMIZATION",
        possible_values = [
            DebugOpt::Initial.to_static_str(),
            Opt::CleanCfg.to_static_str(),
            Opt::ConstantPropagation.to_static_str(),
            Opt::CopyPropagation.to_static_str(),
            Opt::DeadCodeElimination.to_static_str(),
            Opt::RegisterAllocation.to_static_str(),
            DebugOpt::Final.to_static_str(),
        ],
        display_order = 14,
    )]
    debug_optimize_assembly: Vec<DebugOpt>,

    /// Print a newline-separated list of supported optimizations
    #[clap(long = "report-opts", display_order = 15)]
    report_optimizations: bool,

    /// Source files to compile, relative to `source_dir`
    #[clap(value_hint = ValueHint::FilePath)]
    input: Vec<PathBuf>,
}

impl Command {
    fn optimize(&self, optimization: Opt) -> bool {
        match (
            self.optimize_enable.as_ref(),
            self.optimize_disable.as_ref(),
        ) {
            (None, None) => true,
            (None, Some(disable)) => !disable.contains(&optimization),
            (Some(enable), None) => enable.contains(&optimization),
            (Some(_), Some(_)) => unreachable!("Mutual exclusivity guaranteed by `clap`"),
        }
    }

    fn debug_optimize_lir<T: fmt::Display, O: Into<DebugOpt>>(
        &self,
        path: &Path,
        optimization: O,
        data: T,
    ) -> anyhow::Result<()> {
        let optimization = optimization.into();

        if !self.debug_optimize_lir.contains(&optimization) {
            return Ok(());
        }

        self.debug_optimize(path, optimization, data)
    }

    fn debug_optimize_assembly<T: fmt::Display, O: Into<DebugOpt>>(
        &self,
        path: &Path,
        optimization: O,
        data: T,
    ) -> anyhow::Result<()> {
        let optimization = optimization.into();

        if !self.debug_optimize_assembly.contains(&optimization) {
            return Ok(());
        }

        self.debug_optimize(path, optimization, data)
    }

    fn debug_optimize<T: fmt::Display>(
        &self,
        path: &Path,
        optimization: DebugOpt,
        data: T,
    ) -> anyhow::Result<()> {
        // https://github.com/rust-lang/rust/issues/86319
        let mut file_stem = path
            .file_stem()
            .map(OsStr::to_os_string)
            .ok_or_else(|| anyhow!("Expected .xi file, but got {}", path.display()))?;

        file_stem.push("_");
        file_stem.push(optimization.to_static_str());

        let path = path.with_file_name(file_stem);

        self.debug(&path, "dot", data)
    }

    fn debug<T: fmt::Display>(&self, path: &Path, extension: &str, data: T) -> anyhow::Result<()> {
        if self.directory_debug == Path::new("-") {
            println!("{}", data);
            return Ok(());
        }

        self.write(&self.directory_debug.join(path), extension, data)
    }

    fn output<T: fmt::Display>(&self, path: &Path, extension: &str, data: T) -> anyhow::Result<()> {
        if self.directory_output == Path::new("-") {
            println!("{}", data);
            return Ok(());
        }

        self.write(&self.directory_output.join(path), extension, data)
    }

    fn write<T: fmt::Display>(&self, path: &Path, extension: &str, data: T) -> anyhow::Result<()> {
        let path = path.with_extension(extension);
        let mut file = fs::File::create(&path)
            .map(io::BufWriter::new)
            .with_context(|| anyhow!("Failed to create file: {}", path.display()))?;
        write!(file, "{}", data)
            .with_context(|| anyhow!("Failed to write to file: {}", path.display()))?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DebugOpt {
    Initial,
    Opt(Opt),
    Final,
}

impl From<Opt> for DebugOpt {
    fn from(optimization: Opt) -> Self {
        Self::Opt(optimization)
    }
}

impl DebugOpt {
    fn to_static_str(self) -> &'static str {
        match self {
            DebugOpt::Initial => "initial",
            DebugOpt::Opt(optimization) => optimization.to_static_str(),
            DebugOpt::Final => "final",
        }
    }
}

impl str::FromStr for DebugOpt {
    type Err = anyhow::Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "initial" => Ok(DebugOpt::Initial),
            "final" => Ok(DebugOpt::Final),
            _ => Opt::from_str(string).map(DebugOpt::Opt),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Opt {
    LoopInversion,
    ConstantFold,
    FinalClass,
    CleanCfg,
    Inline,
    ConditionalConstantPropagation,
    PartialRedundancyElimination,
    ConstantPropagation,
    CopyPropagation,
    DeadCodeElimination,
    RegisterAllocation,
}

// Need something like https://doc.rust-lang.org/std/mem/fn.variant_count.html
// to make sure array matches up with enum definition. Procedural macro options
// seem too heavyweight for something like this.
const OPTIMIZATIONS: [&str; 11] = [
    Opt::LoopInversion.to_static_str(),
    Opt::ConstantFold.to_static_str(),
    Opt::FinalClass.to_static_str(),
    Opt::CleanCfg.to_static_str(),
    Opt::Inline.to_static_str(),
    Opt::ConditionalConstantPropagation.to_static_str(),
    Opt::PartialRedundancyElimination.to_static_str(),
    Opt::ConstantPropagation.to_static_str(),
    Opt::CopyPropagation.to_static_str(),
    Opt::DeadCodeElimination.to_static_str(),
    Opt::RegisterAllocation.to_static_str(),
];

impl Opt {
    const fn to_static_str(self) -> &'static str {
        match self {
            Opt::LoopInversion => "li",
            Opt::ConstantFold => "cf",
            Opt::FinalClass => "fc",
            Opt::CleanCfg => "clean",
            Opt::Inline => "inl",
            Opt::ConditionalConstantPropagation => "ccp",
            Opt::PartialRedundancyElimination => "pre",
            Opt::ConstantPropagation => "cp",
            Opt::CopyPropagation => "copy",
            Opt::DeadCodeElimination => "dce",
            Opt::RegisterAllocation => "reg",
        }
    }
}

impl str::FromStr for Opt {
    type Err = anyhow::Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "li" => Ok(Opt::LoopInversion),
            "cf" => Ok(Opt::ConstantFold),
            "fc" => Ok(Opt::FinalClass),
            "clean" => Ok(Opt::CleanCfg),
            "inl" => Ok(Opt::Inline),
            "ccp" => Ok(Opt::ConditionalConstantPropagation),
            "pre" => Ok(Opt::PartialRedundancyElimination),
            "cp" => Ok(Opt::ConstantPropagation),
            "copy" => Ok(Opt::CopyPropagation),
            "dce" => Ok(Opt::DeadCodeElimination),
            "reg" => Ok(Opt::RegisterAllocation),
            _ => Err(anyhow!("Unknown optimization {}, ", string)),
        }
    }
}

fn run() -> anyhow::Result<()> {
    pretty_env_logger::init_timed();

    let command = Command::parse();

    if command.report_optimizations {
        for optimization in OPTIMIZATIONS {
            println!("{}", optimization);
        }
        return Ok(());
    }

    for path in &command.input {
        let path = command.directory_source.join(path);

        let tokens = api::lex(&path)?;

        if command.debug_lex {
            command.debug(&path, "lexed", &tokens)?;
        }

        let program = api::parse(&path, tokens)?;

        if command.debug_parse {
            command.debug(&path, "parsed", &program)?;
        }

        let checked = api::check(command.directory_library.as_deref(), &path, program);

        if command.debug_check {
            command.debug(
                &path,
                "typed",
                match &checked {
                    Ok(_) => String::from("Valid Xi Program"),
                    Err(error) => error.to_string(),
                },
            )?;
        }

        let (mut program, mut context) = checked?;

        if command.optimize(Opt::LoopInversion) {
            optimize::invert_loops_ast(&path, &mut program);
        }

        let abi = match command.optimize(Opt::FinalClass) {
            false => xic::Abi::Xi,
            true => xic::Abi::XiFinal,
        };

        let mut hir = api::emit_hir(&mut context, &path, abi, &program);

        if command.optimize(Opt::ConstantFold) {
            hir = hir.map(optimize::fold_constants);
        }

        if command.debug_hir {
            command.debug(&path, "hir", &hir)?;
        }

        let mut lir = hir.map_ref(api::emit_lir);

        if command.optimize(Opt::ConstantFold) {
            lir = lir.map(optimize::fold_constants);
        }

        let mut cfg = lir.map(api::construct_cfg);

        command.debug_optimize_lir(&path, DebugOpt::Initial, &cfg)?;

        if command.optimize(Opt::CleanCfg) {
            cfg = cfg.map_mut(api::clean_cfg);
            command.debug_optimize_lir(&path, DebugOpt::Opt(Opt::CleanCfg), &cfg)?;
        }

        let mut cfg = if command.optimize(Opt::Inline) {
            optimize::inline_functions_lir(cfg)
        } else {
            cfg.map(api::destruct_cfg)
        }
        .map(api::construct_cfg);

        if command.optimize(Opt::Inline) {
            command.debug_optimize_lir(&path, DebugOpt::Opt(Opt::Inline), &cfg)?;
        }

        if command.optimize(Opt::CopyPropagation) {
            cfg = cfg.map_mut(optimize::propagate_copies_lir);
            command.debug_optimize_lir(&path, DebugOpt::Opt(Opt::CopyPropagation), &cfg)?;
        }

        if command.optimize(Opt::ConditionalConstantPropagation) {
            cfg = cfg.map_mut(optimize::propagate_conditional_constants_lir);
            command.debug_optimize_lir(
                &path,
                DebugOpt::Opt(Opt::ConditionalConstantPropagation),
                &cfg,
            )?;
        }

        if command.optimize(Opt::DeadCodeElimination) {
            cfg = cfg.map_mut(optimize::eliminate_dead_code_lir);
            optimize::eliminate_dead_code_functions(&mut cfg);
            command.debug_optimize_lir(&path, DebugOpt::Opt(Opt::DeadCodeElimination), &cfg)?;
        }

        if command.optimize(Opt::PartialRedundancyElimination) {
            cfg = cfg.map_mut(optimize::eliminate_partial_redundancy_lir);
            command.debug_optimize_lir(
                &path,
                DebugOpt::Opt(Opt::PartialRedundancyElimination),
                &cfg,
            )?;
        }

        if command.optimize(Opt::CleanCfg) {
            cfg = cfg.map_mut(api::clean_cfg);
        }

        command.debug_optimize_lir(&path, DebugOpt::Final, &cfg)?;

        let lir = cfg.map(api::destruct_cfg);

        if command.debug_lir {
            command.debug(&path, "lir", &lir)?;
        }

        if command.interpret_ir {
            api::interpret_lir(&lir, io::BufReader::new(io::stdin()), io::stdout())?;
        }

        let abstract_assembly = lir.map_ref(api::tile);

        if command.debug_assembly {
            command.debug(&path, "tiled", &abstract_assembly)?;
        }

        let mut cfg = abstract_assembly.map(api::construct_cfg);

        command.debug_optimize_assembly(&path, DebugOpt::Initial, &cfg)?;

        if command.optimize(Opt::CleanCfg) {
            cfg = cfg.map_mut(api::clean_cfg);
            command.debug_optimize_assembly(&path, DebugOpt::Opt(Opt::CleanCfg), &cfg)?;
        }

        if command.optimize(Opt::ConstantPropagation) {
            cfg = cfg.map_mut(optimize::propagate_constants_assembly);
            command.debug_optimize_assembly(&path, Opt::ConstantPropagation, &cfg)?;
        }

        if command.optimize(Opt::CopyPropagation) {
            cfg = cfg.map_mut(optimize::propagate_copies_assembly);
            command.debug_optimize_assembly(&path, Opt::CopyPropagation, &cfg)?;
        }

        // Register allocation implies dead code elimination, so we don't want to
        // run it redundantly if the user doesn't want debug output for it.
        if command.optimize(Opt::DeadCodeElimination)
            && (!command.optimize(Opt::RegisterAllocation)
                || command
                    .debug_optimize_assembly
                    .contains(&DebugOpt::Opt(Opt::DeadCodeElimination)))
        {
            cfg = cfg.map_mut(|function| {
                let live_variables = analyze::analyze(function);
                optimize::eliminate_dead_code_assembly(&live_variables, function);
            });

            command.debug_optimize_assembly(&path, Opt::DeadCodeElimination, &cfg)?;
        }

        command.debug_optimize_assembly(&path, DebugOpt::Final, &cfg)?;

        let mut assembly = if command.optimize(Opt::RegisterAllocation) {
            cfg.map(api::allocate_linear)
        } else {
            cfg.map(api::destruct_cfg).map_ref(api::allocate_trivial)
        };

        if command.optimize(Opt::CleanCfg) {
            assembly = assembly
                .map(api::construct_cfg)
                .map_mut(api::clean_cfg)
                .map(api::destruct_cfg)
        };

        command.output(&path, "S", assembly.intel())?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let error = match run() {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };

    let error = match error.downcast::<xic::Error>() {
        Ok(error) => error,
        Err(error) => return Err(error),
    };

    if let Some(report) = error.report() {
        let (character_set, color) = match atty::is(atty::Stream::Stderr) {
            true => (ariadne::CharSet::Unicode, true),
            false => (ariadne::CharSet::Ascii, false),
        };

        let report = report
            .with_config(
                ariadne::Config::default()
                    .with_char_set(character_set)
                    .with_color(color),
            )
            .finish();

        let mut cache = xic::data::span::FileCache::default();
        report.eprint(&mut cache)?;
        process::exit(1)
    } else {
        Err(anyhow::Error::from(error))
    }
}
