//! Error reporting.

use std::fmt::Display;

use crate::input_stream::Location;

/// Interface to report errors conveniently.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn warn(&'a mut self) -> ErrorBuilder<'a> {
        ErrorBuilder::new(self, Severity::Warning)
    }

    /// Build error.
    pub fn error(&'a mut self) -> ErrorBuilder<'a> {
        ErrorBuilder::new(self, Severity::Error)
    }

    /// Check if any fatal error occurred.
    pub fn compilation_failed(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ErrorReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for warning in self.warnings.iter() {
            writeln!(f, "Warning at {}:\n\t{}", warning.start, warning.message)?;
        }
        for error in self.errors.iter() {
            writeln!(f, "Error at {}:\n\t{}", error.start, error.message)?;
        }
        writeln!(
            f,
            "{} warning(s), {} error(s)",
            self.warnings.len(),
            self.errors.len()
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
    start: Location,
    end: Location,
}

#[derive(Debug)]
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
    pub fn message(mut self, msg: String) -> ErrorBuilder<'a> {
        self.message = Some(msg);
        self
    }

    /// Set starting location of error. Required.
    pub fn starts_at(mut self, location: Location) -> ErrorBuilder<'a> {
        self.start = Some(location);
        self
    }

    /// Set ending location of error. Defaults to starting location.
    pub fn ends_at(mut self, location: Location) -> ErrorBuilder<'a> {
        self.end = Some(location);
        self
    }

    /// Build error and store it in `ErrorReporter`.
    ///
    /// # Panics
    ///
    /// Panics if not all required fields were set.
    pub fn report(self) {
        let message = self.message.expect("Error message wasn't provided.");
        let start = self
            .start
            .expect("Error starting location wasn't provided.");
        let end = self.end.unwrap_or(start);

        let error = Error {
            message,
            start,
            end,
        };
        match self.severity {
            Severity::Warning => self.reporter.warnings.push(error),
            Severity::Error => self.reporter.errors.push(error),
        }
    }
}

/// How severe is error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Warning,
    Error,
}
