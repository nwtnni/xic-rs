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

    /// Specify where to place generated diagnostic files
    #[structopt(short = "D", parse(from_os_str))]
    output_dir: Option<std::path::PathBuf>,

    /// Source files to compile
    #[structopt(parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

fn run(args: Arguments) -> Result<(), xic::Error> {
    let directory = args.output_dir.unwrap_or_else(|| "".into());
    let lexer = xic::lex::Driver::new(&directory, args.lex_output);
    let parser = xic::parse::Driver::new(&directory, args.parse_output);
    for path in &args.files {
        let tokens = lexer.drive(path)?;
        parser.drive(path, tokens)?;
    }
    Ok(())
}

fn main() {
    let args = Arguments::from_args();
    if let Err(error) = run(args) {
        println!("{}", error);
    }
}
