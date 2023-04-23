//! Parsing stage of the compilation.

mod expression;
mod item;
pub mod operator_expression;
mod statement;

use std::{fmt::Display, path::PathBuf};

pub use expression::*;
pub use item::*;
pub use statement::*;

use thiserror::Error;

use crate::{
    ast::item::{Item, Visibility},
    context::Context,
    input_stream::InputStream,
    path::AbsolutePath,
    item_table::ItemTable,
    lexer::{Lexer, LexerError},
    source::{SourceError, SourceId},
};

/// Interface to compute a [ItemTable] of the whole project.
pub struct Parser {
    pending: Vec<PendingFile>,
    pub context: Context,
}

impl Parser {
    pub fn new(main: PathBuf, context: Context) -> Result<Self, SourceError> {
        Ok(Parser {
            pending: vec![PendingFile::Specific {
                scope: AbsolutePath::new(context.metadata.crate_name.clone()),
                path: main,
            }],
            context,
        })
    }

    /// Parse the whole package.
    pub fn parse(&mut self) -> Result<ItemTable, Vec<ParserError>> {
        let mut table = ItemTable::new();
        let mut errors = Vec::new();
        while let Some(file) = self.pending.pop() {
            let parsed = match file {
                PendingFile::General(path) => self.parse_file(path.clone()),
                PendingFile::Specific { scope, path } => self.parse_file_by_path(scope, path),
            };
            match parsed {
                Ok(parsed) => {
                    self.pending.extend(parsed.pending);
                    table.extend(parsed.item_table);
                }
                Err(err) => {
                    self.pending.extend(err.pending);
                    errors.push(err.inner);
                }
            }
        }

        if errors.is_empty() {
            Ok(table)
        } else {
            Err(errors)
        }
    }

    /// Parse one file at default location.
    pub fn parse_file(&mut self, path: AbsolutePath) -> Result<ParsedFile, ParserErrorExt> {
        let id = self.context.source.lock().unwrap().insert(path.clone())?;
        self.parse_file_by_id(path, id)
    }

    /// Parse one file with specified location.
    pub fn parse_file_by_path(
        &mut self,
        scope: AbsolutePath,
        path: PathBuf,
    ) -> Result<ParsedFile, ParserErrorExt> {
        let id = self.context.source.lock().unwrap().insert_path(path)?;
        self.parse_file_by_id(scope, id)
    }

    fn parse_file_by_id(
        &mut self,
        scope: AbsolutePath,
        id: SourceId,
    ) -> Result<ParsedFile, ParserErrorExt> {
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
            .map_err(|e| e.into())
            .and_then(|parser| parser.parse())
    }
}

/// Interface to parse a single file into [ItemTable].
pub struct FileParser {
    pub item_table: ItemTable,
    pub lexer: Lexer,
    scope: AbsolutePath,
    pending: Vec<PendingFile>,
    pub context: Context,
}

impl FileParser {
    pub fn new(lexer: Lexer, scope: AbsolutePath, context: Context) -> Self {
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
        use crate::Identifier;

        let context = Context::new_test();
        Self {
            item_table: ItemTable::new(),
            lexer: Lexer::new(InputStream::new(src, None), context.clone()),
            scope: AbsolutePath::new(Identifier(String::from("crate"))),
            pending: Vec::new(),
            context,
        }
    }

    pub fn parse(mut self) -> Result<ParsedFile, ParserErrorExt> {
        match self.parse_top_module(self.scope.last().clone()) {
            Ok(module) => {
                self.item_table
                    .declare_anonymous(self.scope.clone(), Item::new(module, Visibility::Public));
                Ok(ParsedFile {
                    item_table: self.item_table,
                    pending: self.pending,
                })
            }
            Err(inner) => Err(ParserErrorExt {
                pending: self.pending,
                inner,
            }),
        }
    }
}

/// Error that has occured during parsing.
///
/// Wrapper over [ParserError] with additional payload of pending files.
#[derive(Debug, Error)]
pub struct ParserErrorExt {
    pending: Vec<PendingFile>,
    inner: ParserError,
}

impl Display for ParserErrorExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T: Into<ParserError>> From<T> for ParserErrorExt {
    fn from(value: T) -> Self {
        Self {
            pending: Vec::default(),
            inner: value.into(),
        }
    }
}

/// Error that has occured during parsing.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("")]
    ParserError,
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
    General(AbsolutePath),
    Specific { scope: AbsolutePath, path: PathBuf },
}
