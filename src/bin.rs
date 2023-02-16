use clap::Parser as ArgParser;
use compiler::{
    Identifier,
    ast::pretty_print::print_table,
    context::{Context, Emit, Metadata},
    parser::Parser,
};
use std::{io::stdout, path::PathBuf, str::FromStr};

#[derive(ArgParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the root file of the crate", value_name = "INPUT")]
    path: PathBuf,
    #[arg(
        long,
        value_name = "NAME",
        help = "Specify the name of the crate being built"
    )]
    crate_name: Option<Identifier>,
    #[arg(long, default_value = "binary")]
    emit: Emit,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let crate_name = match args.crate_name {
        Some(crate_name) => crate_name,
        None => {
            let x = args.path.file_stem().unwrap().to_string_lossy().to_string();
            Identifier::from_str(&x)?
        }
    };
    let context = Context::new(
        args.path.clone(),
        Metadata {
            crate_name,
            emit_type: args.emit,
        },
    )?;
    let mut parser = Parser::new(args.path, context)?;

    let item_table = parser.parse();

    match parser.context.metadata.emit_type {
        Emit::Ast => match &item_table {
            Ok(table) => print_table(stdout(), table)?,
            Err(_) => {
                println!("{}", parser.context.error_reporter);
            }
        },
        Emit::LlvmIr => todo!(),
        Emit::Binary => todo!(),
    };

    Ok(())
}
