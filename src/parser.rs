//! Parsing stage of the compilation.

mod expression;
mod item;
pub mod operator_expression;
mod statement;

use std::{path::PathBuf, sync::Arc};

pub use expression::*;
pub use item::*;
pub use statement::*;

use thiserror::Error;

use crate::{
    ast::{
        item::{Item, ItemKind, Module, Visibility},
        Identifier,
    },
    context::Context,
    input_stream::InputStream,
    item_table::{path::ItemPath, ItemTable},
    lexer::{
        keyword::Keyword,
        punctuation::{BinaryOp, Punctuation, UnaryOp},
        Lexer, LexerError, Token,
    },
    source::{SourceError, SourceMap},
};

/// Interface to compute a [ItemTable] of the whole project.
pub struct Parser {
    /// Path to the root folder.
    source: SourceMap,
    pub context: Arc<Context>,
}

impl Parser {
    pub fn new(root: PathBuf, context: Arc<Context>) -> Result<Self, SourceError> {
        Ok(Parser {
            source: SourceMap::new(root, context.metadata.crate_name.clone())?,
            context,
        })
    }

    /// Parse the whole package.
    pub fn parse(&mut self) -> Result<ItemTable, ParserError> {
        let mut table = ItemTable::new();
        let mut unparsed = vec![ItemPath::new(self.context.metadata.crate_name.clone())];
        while let Some(path) = unparsed.pop() {
            let mut parsed = self.parse_file(path)?;
            for (path, item) in parsed.iter_mut() {
                if let ItemKind::Module(Module::Loadable(ident)) = &mut item.kind {
                    item.kind = ItemKind::Module(Module::Inline(ident.clone()));
                    unparsed.push(path.clone());
                }
            }
            table.extend(parsed);
        }
        Ok(table)
    }

    /// Parse one file.
    pub fn parse_file(&mut self, path: ItemPath) -> Result<ItemTable, ParserError> {
        self.source
            .insert(path.clone())
            .and_then(|src| src.read())
            .map_err(ParserError::SourceError)
            .map(InputStream::new)
            .map(|input| Lexer::new(input, Arc::clone(&self.context)))
            .map(|lexer| FileParser::new(lexer, path, Arc::clone(&self.context)))
            .and_then(|mut parser| parser.parse())
    }
}

/// Interface to parse a single file into [ItemTable].
pub struct FileParser {
    pub item_table: ItemTable,
    pub lexer: Lexer,
    scope: ItemPath,
    pub context: Arc<Context>,
}

impl FileParser {
    pub fn new(lexer: Lexer, scope: ItemPath, context: Arc<Context>) -> Self {
        Self {
            item_table: ItemTable::new(),
            lexer,
            scope,
            context,
        }
    }

    #[cfg(test)]
    pub fn new_test(src: &str) -> Self {
        let context = Arc::new(Context::new_test());
        Self {
            item_table: ItemTable::new(),
            lexer: Lexer::new(InputStream::new(src), Arc::clone(&context)),
            scope: ItemPath::new(Identifier(String::from("crate"))),
            context,
        }
    }

    pub fn parse(&mut self) -> Result<ItemTable, ParserError> {
        let module = self.parse_top_module(self.scope.last().clone())?;
        self.item_table
            .declare_anonymous(self.scope.clone(), Item::new(module, Visibility::Public));
        Ok(self.item_table.clone())
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
    #[error("source error occured: {0}.")]
    SourceError(#[from] SourceError),
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
            self.discard();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if next token is provided keyword and consumes it if so.
    pub fn consume_keyword(&mut self, kw: Keyword) -> Result<bool, ParserError> {
        if self.peek()? == Token::Keyword(kw) {
            self.discard();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if next token is identifier and consumes it if so.
    pub fn consume_identifier(&mut self) -> Result<Option<Identifier>, LexerError> {
        let Token::Identifier(ident) = self.peek()? else { return Ok(None); };
        self.discard();
        Ok(Some(Identifier(ident)))
    }

    /// Checks if next token is unary operator and consumes it if so.
    pub fn consume_unary_operator(&mut self) -> Result<Option<UnaryOp>, LexerError> {
        let Token::Punctuation(punc) = self.peek()? else { return Ok(None); };
        match UnaryOp::try_from(punc) {
            Ok(op) => {
                self.discard();
                Ok(Some(op))
            }
            Err(_) => Ok(None),
        }
    }

    /// Checks if next token is binary operator and consumes it if so.
    pub fn consume_binary_operator(&mut self) -> Result<Option<BinaryOp>, LexerError> {
        let Token::Punctuation(punc) = self.peek()? else { return Ok(None); };
        let Ok(op) = BinaryOp::try_from(punc) else { return Ok(None); };
        self.discard();
        Ok(Some(op))
    }

    /// Check if next token is provided punctuation or error otherwise.
    pub fn expect_punctuation(&mut self, expected: &'static str) -> Result<(), ParserError> {
        let start = self.location;
        let found = self.next()?;
        if found == Token::Punctuation(Punctuation(expected)) {
            Ok(())
        } else {
            self.context
                .error_reporter
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
            self.context
                .error_reporter
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
            self.context
                .error_reporter
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
