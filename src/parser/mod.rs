use thiserror::Error;

use crate::lexer::{Lexer, LexerError, Token, keyword::Keyword, punctuation::Punctuation};

use self::{expressions::*, item::Item, statement::Statement};

pub mod expressions;
mod item;
mod statement;

#[derive(Debug)]
pub struct Ast(Vec<Item>);

impl Ast {
    /// Parse top level of program (file).
    pub fn parse(lexer: &mut Lexer) -> Result<Ast, ParserError> {    
        let mut buffer = Vec::new();
        while !lexer.is_eof() {
            buffer.push(Item::parse(lexer)?);
        }
        Ok(Ast(buffer))
    }
}

impl<'a> Lexer<'a> {
    /// Checks if next token is provided punctuation and consumes it if so.
    /// 
    /// # Returns
    /// 
    /// Returns `true` if provided punctuation matches.
    fn consume_punctuation(&mut self, punc: &'static str) -> Result<bool, ParserError> {
        if self.peek()? == Token::Punctuation(Punctuation(punc)) {
            let _ = self.next();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if next token is provided punctuation or error otherwise.
    fn expect_punctuation(&mut self, expected: &'static str) -> Result<(), ParserError> {
        let start = self.location;
        let found = self.next()?;
        if found == Token::Punctuation(Punctuation(expected)) {
            Ok(())
        } else {
            self.error_reporter.error()
                .message(format!("Expected punctuation `{expected}`, found {found:?}"))
                .starts_at(start)
                .ends_at(self.location)
                .report();
            Err(UnexpectedTokenError::TokenMismatch.into())
        }
    }

    /// Check if next token is provided punctuation or error otherwise.
    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), ParserError> {
        let start = self.location;
        let found = self.next()?;
        if found == Token::Keyword(keyword) {
            Ok(())
        } else {
            self.error_reporter.error()
                .message(format!("Expected keyword `{keyword}`, found {found:?}"))
                .starts_at(start)
                .ends_at(self.location)
                .report();
            Err(UnexpectedTokenError::TokenMismatch.into())
        }
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
