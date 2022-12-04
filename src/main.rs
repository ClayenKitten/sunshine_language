use std::{fs::read_to_string, path::PathBuf};

use parser::Ast;
use clap::Parser;
use lexer::Lexer;

mod parser;
mod input_stream;
mod lexer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let program = read_to_string(args.path)?;
    let mut lexer = Lexer::new(&program);
    let mut ast = Ast::parse(&mut lexer);
    
    println!("{:#?}", ast);

    Ok(())
}
