use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use compiler::{lexer::Lexer, parser::Ast};

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
    let ast = Ast::parse(&mut lexer);
    
    println!("{:#?}", ast);
    println!("{}", lexer.error_reporter());

    Ok(())
}
