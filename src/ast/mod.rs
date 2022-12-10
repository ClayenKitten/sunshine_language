use thiserror::Error;

use crate::lexer::{Lexer, LexerError, Token};

use self::{expressions::*, item::Module};

pub mod expressions;
mod item;
mod statement;

#[derive(Debug)]
pub struct Ast(Module);

impl Ast {
    /// Parse top level of program (file).
    pub fn parse(lexer: &mut Lexer) -> Result<Ast, ParserError> {    
        Module::parse_toplevel(lexer)
            .map(|module| Ast(module))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnexpectedTokenError {
    #[error("unexpected token: ")]
    UnexpectedToken(Token),
    #[error("token mismatch")]
    TokenMismatch,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParserError {
    #[error(transparent)]
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Lexer error occured: {0}.")]
    LexerError(#[from] LexerError),
}
