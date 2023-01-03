use std::{path::PathBuf, sync::{Mutex, Arc}};
use clap::Parser as ArgParser;
use compiler::{parser::Parser, context::{Context, Metadata}, error::ErrorReporter};

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
    let crate_name = args.crate_name
        .unwrap_or_else(|| args.path.file_stem().unwrap().to_string_lossy().to_string());
    let context = Context {
        metadata: Metadata { crate_name },
        error_reporter: Mutex::new(ErrorReporter::new()),
    };
    let mut parser = Parser::new(args.path, Arc::new(context));
    
    let symbol_table = parser.parse();
    match symbol_table {
        Ok(symbol_table) => println!("{}", symbol_table),
        Err(error) => println!("{}", error),
    }

    println!("{}", parser.context.error_reporter.lock().unwrap());
    Ok(())
}
