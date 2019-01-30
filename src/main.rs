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
    for path in &args.files {
        let source = std::fs::read_to_string(path)?; 
        let lexer = xic::Lexer::new(&source);
        println!("\n\n{:?}\n", path);
        for spanned in lexer {
            let (start, token, _) = spanned?;
            println!("{} {}", start, token);
        }
    }
    Ok(())
}
