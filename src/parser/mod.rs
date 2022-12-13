//! Parsing stage of the compilation.

mod expression;
mod item;
pub mod shunting_yard;
mod statement;

pub use expression::*;
pub use item::*;
pub use statement::*;

use thiserror::Error;

use crate::{
    ast::{item::Item, Identifier, Visibility},
    error::ErrorReporter,
    lexer::{keyword::Keyword, punctuation::Punctuation, Lexer, LexerError, Token},
    symbol_table::{Path, SymbolTable},
};

/// Interface to parse a single file into [SymbolTable].
pub struct FileParser<'s> {
    pub symbol_table: SymbolTable,
    pub lexer: Lexer<'s>,
    scope: Path,
    pub error_reporter: ErrorReporter,
}

impl<'s> FileParser<'s> {
    pub fn new(lexer: Lexer<'s>) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            lexer,
            scope: Path::new(),
            error_reporter: ErrorReporter::new(),
        }
    }

    pub fn parse(&mut self) -> Result<SymbolTable, ParserError> {
        let module = self.parse_top_module(Identifier(String::from("crate")))?;
        self.symbol_table
            .declare(self.scope.clone(), Item::new(module, Visibility::Public));
        Ok(self.symbol_table.clone())
    }
}

/// Error that has occured during parsing.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParserError {
    #[error(transparent)]
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("expected expression")]
    ExpectedExpression,
    #[error("unclosed parenthesis")]
    UnclosedParenthesis,
    #[error("Lexer error occured: {0}.")]
    LexerError(#[from] LexerError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnexpectedTokenError {
    #[error("unexpected token: ")]
    UnexpectedToken(Token),
    #[error("token mismatch")]
    TokenMismatch,
}

impl<'s> Lexer<'s> {
    /// Check if the following token is provided punctuation without advancing.
    pub fn peek_punctuation(&mut self, punc: &'static str) -> bool {
        let Ok(token) = self.peek() else { return false; };
        token == Token::Punctuation(Punctuation(punc))
    }

    /// Checks if next token is provided punctuation and consumes it if so.
    ///
    /// # Returns
    ///
    /// Returns `true` if provided punctuation matches.
    pub fn consume_punctuation(&mut self, punc: &'static str) -> Result<bool, ParserError> {
        if self.peek()? == Token::Punctuation(Punctuation(punc)) {
            let _ = self.next();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if next token is provided keyword and consumes it if so.
    pub fn consume_keyword(&mut self, kw: Keyword) -> Result<bool, ParserError> {
        if self.peek()? == Token::Keyword(kw) {
            let _ = self.next();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if next token is identifier and consumes it if so.
    pub fn consume_identifier(&mut self) -> Result<Option<Identifier>, LexerError> {
        let token = self.peek()?;
        if let Token::Identifier(ident) = token {
            let _ = self.next();
            Ok(Some(Identifier(ident)))
        } else {
            Ok(None)
        }
    }

    /// Checks if next token is unary operator and consumes it if so.
    pub fn consume_unary_operator(&mut self) -> Result<Option<Punctuation>, LexerError> {
        match self.peek()? {
            Token::Punctuation(punc) if punc.is_unary_operator() => {
                self.discard();
                Ok(Some(punc))
            }
            _ => Ok(None)
        }
    }

    /// Checks if next token is binary operator and consumes it if so.
    pub fn consume_binary_operator(&mut self) -> Result<Option<Punctuation>, LexerError> {
        match self.peek()? {
            Token::Punctuation(punc) if punc.is_binary_operator() => {
                self.discard();
                Ok(Some(punc))
            }
            _ => Ok(None)
        }
    }

    /// Check if next token is provided punctuation or error otherwise.
    pub fn expect_punctuation(&mut self, expected: &'static str) -> Result<(), ParserError> {
        let start = self.location;
        let found = self.next()?;
        if found == Token::Punctuation(Punctuation(expected)) {
            Ok(())
        } else {
            self.error_reporter
                .error()
                .message(format!(
                    "Expected punctuation `{expected}`, found {found:?}"
                ))
                .starts_at(start)
                .ends_at(self.location)
                .report();
            Err(UnexpectedTokenError::TokenMismatch.into())
        }
    }

    /// Check if next token is provided punctuation or error otherwise.
    pub fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), ParserError> {
        let start = self.location;
        let found = self.next()?;
        if found == Token::Keyword(keyword) {
            Ok(())
        } else {
            self.error_reporter
                .error()
                .message(format!("Expected keyword `{keyword}`, found {found:?}"))
                .starts_at(start)
                .ends_at(self.location)
                .report();
            Err(UnexpectedTokenError::TokenMismatch.into())
        }
    }

    /// Check if next token is identifier or error otherwise.
    pub fn expect_identifier(&mut self) -> Result<Identifier, ParserError> {
        let start = self.location;
        let found = self.next()?;
        if let Token::Identifier(ident) = found {
            Ok(Identifier(ident))
        } else {
            self.error_reporter
                .error()
                .message(format!("Expected identifier, found {found:?}"))
                .starts_at(start)
                .ends_at(self.location)
                .report();
            Err(UnexpectedTokenError::TokenMismatch.into())
        }
    }
}
