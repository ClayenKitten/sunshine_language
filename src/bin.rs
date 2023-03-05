use clap::Parser as ArgParser;
use compiler::{
    ast::pretty_print::print_table,
    context::{Context, Emit, Metadata},
    hir::HirData,
    parser::Parser,
    Identifier,
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
        Emit::Hir => match item_table {
            Ok(item_table) => {
                match HirData::translate(item_table) {
                    Ok(hir) => println!("{:#?}", hir),
                    Err(err) => println!("{}", err),
                };
            }
            Err(_) => {
                println!("{}", parser.context.error_reporter);
            }
        },
        Emit::LlvmIr => todo!(),
        Emit::Binary => todo!(),
    };

    Ok(())
}
