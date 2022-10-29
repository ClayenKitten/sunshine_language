use thiserror::Error;

use crate::lexer::{TokenStream, LexerError, Token, TokenKind, keyword::Keyword, punctuation::Punctuation};

use self::{expressions::*, item::Item};

pub mod expressions;
mod item;

#[derive(Debug)]
pub struct Ast(Vec<Item>);

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Item(Item),
    ExpressionStatement(Expression),
    LetStatement(LetStatement),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    WithBlock(ExpressionWithBlock),
    WithoutBlock(ExpressionWithoutBlock),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionWithBlock {
    Block(Vec<Statement>),
    If(If),
    While(While),
    For(For),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionWithoutBlock {
    Identifier(Identifier),
    Literal(Literal),
    Assignment(Assignment),
    FunctionCall(FunctionCall),
}

impl Ast {
    pub fn parse(lexer: &mut TokenStream) -> Result<Ast, ParserError> {    
        todo!();
    }
}

impl TokenStream {
    fn expect_punctuation(&mut self, punc: &'static str) -> Result<(), UnexpectedTokenError> {
        match self.next()? {
            Some(Token::Punctuation(got)) if got == Punctuation::new(punc) => Ok(()),
            Some(got) => Err(UnexpectedTokenError::TokenMismatch {
                expected: Token::Punctuation(Punctuation::new(punc)),
                got
            }),
            None => Err(UnexpectedTokenError::Eof),
        }
    }

    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), UnexpectedTokenError> {
        match self.next()? {
            Some(Token::Keyword(got)) if got == keyword => Ok(()),
            Some(got) => Err(UnexpectedTokenError::TokenMismatch {
                expected: Token::Keyword(keyword),
                got
            }),
            None => Err(UnexpectedTokenError::Eof),
        }
    }

    /// Returns error if EOF achieved.
    fn next_some(&mut self) -> Result<Token, ParserError> {
        match self.next()? {
            Some(token) => Ok(token),
            None => Err(ParserError::UnexpectedEof),
        }
    }

    fn next_expected_kind(&mut self, expected: TokenKind) -> Result<Token, UnexpectedTokenError> {
        match self.next()? {
            Some(token) if expected == (&token).into() => Ok(token),
            Some(token) => Err(UnexpectedTokenError::TypeMismatch { expected, got: token.into() }),
            None => Err(UnexpectedTokenError::Eof),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnexpectedTokenError {
    #[error("unexpected token")]
    UnexpectedToken(Token),
    #[error("token mismatch")]
    TokenMismatch { expected: Token, got: Token },
    #[error("token type mismatch")]
    TypeMismatch { expected: TokenKind, got: TokenKind },
    #[error("unexpected end of file")]
    Eof,
    #[error("{0}")]
    LexerError(#[from] LexerError),
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParserError {
    #[error("Invalid top-level token.")]
    InvalidTopLevel,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Lexer error occured: {0}.")]
    LexerError(#[from] LexerError),
}
