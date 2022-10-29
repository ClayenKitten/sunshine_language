use std::{fs::read_to_string, path::PathBuf};

use ast::Ast;
use clap::Parser;
use lexer::TokenStream;

mod ast;
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
    let mut lexer = TokenStream::new(&program);
    let mut ast = Ast::parse(&mut lexer);
    
    println!("{:#?}", ast);

    Ok(())
}
