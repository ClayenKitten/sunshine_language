use std::{error::Error, sync::Arc};

use crate::{input_stream::Location, lexer::Lexer, parser::FileParser, source::SourceId};

use super::ErrorReporter;

/// Error that may be reported.
pub trait ReportableError: Error {
    fn severity(&self) -> Severity;
    fn span(&self) -> ErrorSpan;
}

/// How severe is the error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// User attention requested.
    Warn,
    /// Compilation failed.
    Deny,
}

/// Location of the error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorSpan {
    pub source: Option<SourceId>,
    pub start: Location,
    pub end: Location,
}

/// A struct that has all information required to report a error.
pub trait ReportProvider {
    /// Returns a reference to the error reporter.
    fn error_reporter(&self) -> Arc<ErrorReporter>;
    /// Returns current location of the cursor.
    fn location(&self) -> Location;
    /// Returns id of the file being parsed, if any.
    fn source(&self) -> Option<SourceId>;
}

impl ReportProvider for FileParser {
    fn error_reporter(&self) -> Arc<ErrorReporter> {
        self.context.error_reporter.clone()
    }

    fn location(&self) -> Location {
        self.lexer.location()
    }

    fn source(&self) -> Option<SourceId> {
        self.lexer.source()
    }
}

impl ReportProvider for Lexer {
    fn error_reporter(&self) -> Arc<ErrorReporter> {
        self.context.error_reporter.clone()
    }

    fn location(&self) -> Location {
        self.input.location()
    }

    fn source(&self) -> Option<SourceId> {
        self.input.source()
    }
}
