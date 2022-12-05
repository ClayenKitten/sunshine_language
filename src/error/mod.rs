//! Error reporting

use std::cmp::Ordering;

/// Interface to report errors conveniently.
pub struct ErrorReporter {
    /// Valid code that looks suspicious to compiler.
    warnings: Vec<Error>,
    /// Invalid code that doesn't allow compilation to proceed.
    errors: Vec<Error>,
}

impl<'a> ErrorReporter {
    /// Create new ErrorReporter.
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Build warning.
    pub fn warn(&'a mut self, line: usize, column: usize) -> ErrorBuilder<'a> {
        ErrorBuilder::new(self, Severity::Warning)
    }

    /// Build error.
    pub fn error(&'a mut self, line: usize, column: usize) -> ErrorBuilder<'a> {
        ErrorBuilder::new(self, Severity::Error)
    }
}

pub struct Error {
    message: String,
    start: Location,
    end: Location,
}

pub struct ErrorBuilder<'a> {
    reporter: &'a mut ErrorReporter,
    severity: Severity,
    message: Option<String>,
    start: Option<Location>,
    end: Option<Location>,
}

impl<'a> ErrorBuilder<'a> {
    fn new(reporter: &'a mut ErrorReporter, severity: Severity) -> Self {
        Self {
            reporter,
            severity,
            message: None,
            start: None,
            end: None,
        }
    }

    /// Set message of error. Required.
    pub fn message(&mut self, msg: String) {
        self.message = Some(msg);
    }

    /// Set starting location of error. Required.
    pub fn starts_at(&mut self, line: usize, column: usize) {
        self.start = Some(Location { line, column })
    }

    /// Set ending location of error. Defaults to starting location.
    pub fn ends_at(&mut self, line: usize, column: usize) {
        self.end = Some(Location { line, column })
    }

    /// Build error and store it in `ErrorReporter`.
    /// 
    /// # Panics
    /// 
    /// Panics if not all required fields were set.
    pub fn report(self) {
        let message = self.message.expect("Error message wasn't provided.");
        let start = self.start.expect("Error starting location wasn't provided.");
        let end = self.end.unwrap_or(start);

        let error = Error { message, start, end };
        match self.severity {
            Severity::Warning => self.reporter.warnings.push(error),
            Severity::Error => self.reporter.errors.push(error),
        }
    }
}

#[derive(Debug, Clone, Copy,PartialEq, Eq, Ord)]
struct Location {
    line: usize,
    column: usize,
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.line.cmp(&other.line) {
            Ordering::Equal => {
                self.column.cmp(&other.column).reverse()
            },
            ord => ord.reverse(),
        })
    }
}

/// How severe is error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Warning,
    Error,
}
