//! Parsing stage of the compilation.

mod expression;
mod item;
pub mod operator_expression;
mod statement;

use std::path::PathBuf;

pub use expression::*;
pub use item::*;
pub use statement::*;

use thiserror::Error;

use crate::{
    ast::item::{Item, Visibility},
    context::Context,
    input_stream::InputStream,
    item_table::{path::ItemPath, ItemTable},
    lexer::{Lexer, LexerError, Token},
    source::{SourceError, SourceId},
};

use self::operator_expression::infix::AssignmentError;

/// Interface to compute a [ItemTable] of the whole project.
pub struct Parser {
    pending: Vec<PendingFile>,
    pub context: Context,
}

impl Parser {
    pub fn new(main: PathBuf, context: Context) -> Result<Self, SourceError> {
        Ok(Parser {
            pending: vec![PendingFile::Specific {
                scope: ItemPath::new(context.metadata.crate_name.clone()),
                path: main,
            }],
            context,
        })
    }

    /// Parse the whole package.
    pub fn parse(&mut self) -> Result<ItemTable, ParserError> {
        let mut table = ItemTable::new();
        while let Some(file) = self.pending.pop() {
            let parsed = match file {
                PendingFile::General(path) => self.parse_file(path.clone())?,
                PendingFile::Specific { scope, path } => self.parse_file_by_path(scope, path)?,
            };
            self.pending.extend(parsed.pending);
            table.extend(parsed.item_table);
        }
        Ok(table)
    }

    /// Parse one file at default location.
    pub fn parse_file(&mut self, path: ItemPath) -> Result<ParsedFile, ParserError> {
        let id = self.context.source.lock().unwrap().insert(path.clone())?;
        self.parse_file_by_id(path, id)
    }

    /// Parse one file with specified location.
    pub fn parse_file_by_path(
        &mut self,
        scope: ItemPath,
        path: PathBuf,
    ) -> Result<ParsedFile, ParserError> {
        let id = self.context.source.lock().unwrap().insert_path(path)?;
        self.parse_file_by_id(scope, id)
    }

    fn parse_file_by_id(
        &mut self,
        scope: ItemPath,
        id: SourceId,
    ) -> Result<ParsedFile, ParserError> {
        self.context
            .source
            .lock()
            .unwrap()
            .get(id)
            .read()
            .map_err(ParserError::SourceError)
            .map(|s| InputStream::new(s, Some(id)))
            .map(|input| Lexer::new(input, self.context.clone()))
            .map(|lexer| FileParser::new(lexer, scope, self.context.clone()))
            .and_then(|parser| parser.parse())
    }
}

/// Interface to parse a single file into [ItemTable].
pub struct FileParser {
    pub item_table: ItemTable,
    pub lexer: Lexer,
    scope: ItemPath,
    pending: Vec<PendingFile>,
    pub context: Context,
}

impl FileParser {
    pub fn new(lexer: Lexer, scope: ItemPath, context: Context) -> Self {
        Self {
            item_table: ItemTable::new(),
            lexer,
            scope,
            pending: Vec::new(),
            context,
        }
    }

    #[cfg(test)]
    pub fn new_test(src: &str) -> Self {
        use crate::ast::Identifier;

        let context = Context::new_test();
        Self {
            item_table: ItemTable::new(),
            lexer: Lexer::new(InputStream::new(src, None), context.clone()),
            scope: ItemPath::new(Identifier(String::from("crate"))),
            pending: Vec::new(),
            context,
        }
    }

    pub fn parse(mut self) -> Result<ParsedFile, ParserError> {
        let module = self.parse_top_module(self.scope.last().clone())?;
        self.item_table
            .declare_anonymous(self.scope.clone(), Item::new(module, Visibility::Public));
        Ok(ParsedFile {
            item_table: self.item_table,
            pending: self.pending,
        })
    }

    pub fn source(&self) -> Option<SourceId> {
        self.lexer.source()
    }
}

/// Error that has occured during parsing.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("unexpected token: `{0:?}`")]
    UnexpectedToken(Token),
    #[error(transparent)]
    AssignmentError(#[from] AssignmentError),
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("expected expression")]
    ExpectedExpression,
    #[error("unclosed parenthesis")]
    UnclosedParenthesis,
    #[error("lexer error occured: {0}")]
    LexerError(#[from] LexerError),
    #[error("source error occured: {0}")]
    SourceError(#[from] SourceError),
}

/// Result of the file parse.
pub struct ParsedFile {
    pub item_table: ItemTable,
    pub pending: Vec<PendingFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PendingFile {
    General(ItemPath),
    Specific { scope: ItemPath, path: PathBuf },
}
