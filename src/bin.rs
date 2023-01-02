use std::path::PathBuf;
use clap::Parser as ArgParser;
use compiler::parser::Parser;

#[derive(ArgParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help="Path to the root file of the crate", value_name="INPUT")]
    path: PathBuf,
    #[arg(long, value_name="NAME", help="Specify the name of the crate being built")]
    crate_name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let parser = Parser::new(args.path);
    let (symbol_table, error_reporter) = parser.parse();
    match symbol_table {
        Ok(symbol_table) => println!("{}", symbol_table),
        Err(error) => println!("{}", error),
    }
    println!("{}", error_reporter);

    Ok(())
}
