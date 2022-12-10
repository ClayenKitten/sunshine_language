use thiserror::Error;

use crate::lexer::{LexerError, Token};

use self::{expressions::*, item::Module};

pub mod expressions;
pub mod item;
pub mod statement;

#[derive(Debug)]
pub struct Ast(pub Module);

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
