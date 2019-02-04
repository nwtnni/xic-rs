use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "xic", about = "Compiler for the Xi programming language.")]
struct Arguments {
    /// Generate output from lexical analysis
    #[structopt(short = "l", long = "lex")]
    lex_output: bool,  

    /// Specify where to place generated diagnostic files
    #[structopt(short = "D", parse(from_os_str))]
    output_dir: Option<std::path::PathBuf>,

    /// Source files to compile
    #[structopt(parse(from_os_str))]
    files: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), xic::Error> {
    let args = Arguments::from_args();
    let directory = args.output_dir.unwrap_or_else(|| "".into());
    let lexer = xic::lex::Driver::new(&directory, args.lex_output);
    for path in &args.files {
        lexer.drive(path)?;
    }
    Ok(())
}
