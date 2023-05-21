//! Parsing stage of the compilation.

mod expression;
mod item;
pub mod operator_expression;
mod statement;

use std::path::PathBuf;

pub use expression::*;
pub use item::*;
pub use statement::*;

use crate::{
    ast::item::{Item, Visibility},
    context::Context,
    error::{CompilerError, ReportProvider},
    input_stream::InputStream,
    item_table::ItemTable,
    lexer::Lexer,
    path::AbsolutePath,
    source::{SourceError, SourceId},
    util::Span,
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
    pub fn parse(&mut self) -> Result<ItemTable, Vec<CompilerError>> {
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
                    errors.push(err);
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
    pub fn parse_file(&mut self, path: AbsolutePath) -> Result<ParsedFile, CompilerError> {
        let id = self.context.source.lock().unwrap().insert(path.clone())?;
        self.parse_file_by_id(path, id)
    }

    /// Parse one file with specified location.
    pub fn parse_file_by_path(
        &mut self,
        scope: AbsolutePath,
        path: PathBuf,
    ) -> Result<ParsedFile, CompilerError> {
        let id = self.context.source.lock().unwrap().insert_path(path)?;
        self.parse_file_by_id(scope, id)
    }

    fn parse_file_by_id(
        &mut self,
        scope: AbsolutePath,
        id: SourceId,
    ) -> Result<ParsedFile, CompilerError> {
        let mut source_map = self.context.source.lock().unwrap();
        let file = source_map.get(id).read()?;
        let stream = InputStream::new(file, Some(id));
        let lexer = Lexer::new(stream, self.context.clone());
        let parser = FileParser::new(lexer, scope, self.context.clone());

        parser.parse().map_err(|(err, pending)| {
            self.pending.extend(pending);
            err
        })
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

    pub fn parse(mut self) -> Result<ParsedFile, (CompilerError, Vec<PendingFile>)> {
        let start = self.location();
        match self.parse_top_module(self.scope.last().clone()) {
            Ok(module) => {
                let item = Item::new(
                    module,
                    Span {
                        source: self.source(),
                        start,
                        end: self.location(),
                    },
                    Visibility::Public,
                );
                self.item_table.declare_anonymous(self.scope.clone(), item);
                Ok(ParsedFile {
                    item_table: self.item_table,
                    pending: self.pending,
                })
            }
            Err(err) => Err((err, self.pending)),
        }
    }
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
