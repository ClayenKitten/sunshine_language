//! Parsing stage of the compilation.

mod expression;
mod item;
pub mod shunting_yard;
mod statement;

use std::{path::PathBuf, sync::{Mutex, Arc}};

pub use expression::*;
pub use item::*;
pub use statement::*;

use thiserror::Error;

use crate::{
    ast::{item::Item, Identifier, Visibility},
    error::ErrorReporter,
    lexer::{keyword::Keyword, punctuation::Punctuation, Lexer, LexerError, Token},
    symbol_table::{Path, SymbolTable}, input_stream::InputStream,
};

/// Interface to compute a [SymbolTable] of the whole project.
pub struct Parser {
    /// Path to the root file.
    root: PathBuf,
    error_reporter: Arc<Mutex<ErrorReporter>>,
}

impl Parser {
    pub fn new(root: PathBuf) -> Self {
        Parser {
            root,
            error_reporter: Arc::new(Mutex::new(ErrorReporter::new())),
        }
    }

    /// Parse the whole package.
    pub fn parse(mut self) -> (Result<SymbolTable, ParserError>, ErrorReporter) {
        let table = self.parse_file(&self.root.clone());
        let error_reporter = Arc::try_unwrap(self.error_reporter)
            .expect("as all parsing processes ended, no other references are expected to exist")
            .into_inner()
            .expect("poisoning shouldn't have happened");
        (table, error_reporter)
    }

    fn parse_file(&mut self, path: &std::path::Path) -> Result<SymbolTable, ParserError> {
        std::fs::read_to_string(path)
            .map_err(|e| ParserError::IoError(e))
            .map(|src| InputStream::new(src))
            .map(|input| Lexer::new(input, Arc::clone(&self.error_reporter)))
            .map(|lexer| FileParser::new(lexer, Arc::clone(&self.error_reporter)))
            .and_then(|mut parser| parser.parse())
    }
}

/// Interface to parse a single file into [SymbolTable].
pub struct FileParser {
    pub symbol_table: SymbolTable,
    pub lexer: Lexer,
    scope: Path,
    pub error_reporter: Arc<Mutex<ErrorReporter>>,
}

impl FileParser {
    pub fn new(lexer: Lexer, error_reporter: Arc<Mutex<ErrorReporter>>) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            lexer,
            scope: Path::new(),
            error_reporter,
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
#[derive(Debug, Error)]
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
    #[error("io error occured: {0}.")]
    IoError(#[from] std::io::Error)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnexpectedTokenError {
    #[error("unexpected token: ")]
    UnexpectedToken(Token),
    #[error("token mismatch")]
    TokenMismatch,
}

impl Lexer {
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
                .lock()
                .unwrap()
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
                .lock()
                .unwrap()
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
                .lock()
                .unwrap()
                .error()
                .message(format!("Expected identifier, found {found:?}"))
                .starts_at(start)
                .ends_at(self.location)
                .report();
            Err(UnexpectedTokenError::TokenMismatch.into())
        }
    }
}
