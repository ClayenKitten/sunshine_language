use std::{fs::read_to_string, path::PathBuf};

use clap::Parser as ArgParser;
use compiler::{lexer::Lexer, ast::Ast, input_stream::InputStream, parser::Parser};

#[derive(ArgParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let program = read_to_string(args.path)?;
    let input = InputStream::new(&program);
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    println!("{:#?}", ast);
    println!("{}", parser.error_reporter);

    Ok(())
}
