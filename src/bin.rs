use std::{path::PathBuf, sync::{Mutex, Arc}, io::stdout};
use clap::Parser as ArgParser;
use compiler::{parser::Parser, context::{Context, Metadata, Emit}, error::ErrorReporter, ast::pretty_print::print_table};

#[derive(ArgParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help="Path to the root file of the crate", value_name="INPUT")]
    path: PathBuf,
    #[arg(long, value_name="NAME", help="Specify the name of the crate being built")]
    crate_name: Option<String>,
    #[arg(long, default_value = "binary")]
    emit: Emit,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let crate_name = args.crate_name
        .unwrap_or_else(|| args.path.file_stem().unwrap().to_string_lossy().to_string());
    let context = Context {
        metadata: Metadata { crate_name, emit_type: args.emit },
        error_reporter: Mutex::new(ErrorReporter::new()),
    };
    let mut parser = Parser::new(args.path, Arc::new(context));
    
    let symbol_table = parser.parse();

    println!("{}", parser.context.error_reporter.lock().unwrap());

    match parser.context.metadata.emit_type {
        Emit::Ast => print_table(&mut stdout(), &symbol_table?)?,
        Emit::LlvmIr => todo!(),
        Emit::Binary => todo!(),
    };

    Ok(())
}
