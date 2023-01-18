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
    lexer::{keyword::Keyword, punctuation::Punctuation, Lexer, LexerError, Token},
};

/// Interface to compute a [ItemTable] of the whole project.
pub struct Parser {
    /// Path to the root file.
    root: PathBuf,
    pub context: Arc<Context>,
}

impl Parser {
    pub fn new(root: PathBuf, context: Arc<Context>) -> Self {
        Parser { root, context }
    }

    /// Parse the whole package.
    pub fn parse(&mut self) -> Result<ItemTable, ParserError> {
        self.parse_file_recursive(&self.root.clone())
    }

    /// Parse file and inline all its loadable modules.
    fn parse_file_recursive(&mut self, path: &std::path::Path) -> Result<ItemTable, ParserError> {
        let mut table = self.parse_file(path)?;
        let mut modules = Vec::<PathBuf>::new();
        for (path, item) in table.iter_mut() {
            if let ItemKind::Module(Module::Loadable(ident)) = &mut item.kind {
                item.kind = ItemKind::Module(Module::Inline(ident.clone()));
                let path = path.clone();
                modules.push(self.submodule_path(path));
            }
        }
        for module in modules {
            table.extend(self.parse_file_recursive(&module)?);
        }
        Ok(table)
    }

    fn parse_file(&mut self, path: &std::path::Path) -> Result<ItemTable, ParserError> {
        std::fs::read_to_string(path)
            .map_err(ParserError::IoError)
            .map(InputStream::new)
            .map(|input| Lexer::new(input, Arc::clone(&self.context)))
            .map(|lexer| FileParser::new(lexer, Arc::clone(&self.context)))
            .and_then(|mut parser| parser.parse())
    }

    fn submodule_path(&self, parent: ItemPath) -> PathBuf {
        let mut root_folder = {
            let mut root = self.root.clone();
            root.pop();
            root
        };
        let parent = parent.into_path_buf();
        root_folder.extend(parent.iter());
        root_folder.set_extension("sun");
        root_folder
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
    pub fn new(lexer: Lexer, context: Arc<Context>) -> Self {
        Self {
            item_table: ItemTable::new(),
            lexer,
            scope: ItemPath::new(Identifier(context.metadata.crate_name.clone())),
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
    #[error("io error occured: {0}.")]
    IoError(#[from] std::io::Error),
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
            _ => Ok(None),
        }
    }

    /// Checks if next token is binary operator and consumes it if so.
    pub fn consume_binary_operator(&mut self) -> Result<Option<Punctuation>, LexerError> {
        match self.peek()? {
            Token::Punctuation(punc) if punc.is_binary_operator() => {
                self.discard();
                Ok(Some(punc))
            }
            _ => Ok(None),
        }
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
