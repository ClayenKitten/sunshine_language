use std::sync::Arc;

use crate::{
    error::error_reporter::ErrorReporter, input_stream::Location, lexer::Lexer, parser::FileParser,
    source::SourceId,
};

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
