use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "xic", about = "Compiler for the Xi programming language.")]
struct Arguments {
    /// Generate output from lexical analysis
    #[structopt(short = "l", long = "lex")]
    lex_output: bool,  

    /// Generate output from syntactic analysis
    #[structopt(short = "p", long = "parse")]
    parse_output: bool,

    /// Generate output from semantic analysis
    #[structopt(short = "t", long = "typecheck")]
    type_output: bool,

    /// Generate output from emitted IR
    #[structopt(short = "g", long = "irgen")]
    ir_output: bool,

    /// Disable optimizations
    #[structopt(short = "O")]
    disable_opt: bool,

    /// Specify where to place generated diagnostic files
    #[structopt(short = "D", parse(from_os_str))]
    output_dir: Option<std::path::PathBuf>,

    /// Specify where to search for source files
    #[structopt(long = "sourcepath", parse(from_os_str))]
    source_dir: Option<std::path::PathBuf>,

    /// Specify where to search for library files
    #[structopt(long = "libpath", parse(from_os_str))]
    lib_dir: Option<std::path::PathBuf>,

    /// Source files to compile, relative to `source_dir`
    #[structopt(parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

fn run(args: Arguments) -> Result<(), xic::Error> {
    let directory = args.output_dir.unwrap_or_else(|| "".into());
    let source = args.source_dir.unwrap_or_else(|| "".into());
    let lexer = xic::lex::Driver::new(&directory, args.lex_output);
    let parser = xic::parse::Driver::new(&directory, args.parse_output);
    let checker = xic::check::Driver::new(&directory, args.type_output, args.lib_dir.as_ref());
    let emitter = xic::emit::Driver::new(&directory, args.ir_output, !args.disable_opt);
    for path in &args.files {
        let path = source.join(path);
        let tokens = lexer.drive(&path)?;
        let program = parser.drive(&path, tokens)?;
        let env = checker.drive(&path, &program)?;
        let _lir = emitter.drive(&path, &program, &env)?;
    }
    Ok(())
}

fn main() {
    let args = Arguments::from_args();
    if let Err(error) = run(args) {
        println!("{}", error);
    }
}
