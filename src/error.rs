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

impl ErrorReporter {
    /// Create new ErrorReporter.
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Build error.
    pub fn error(&mut self, message: impl ToString, start: Location, end: Location) {
        let error = Error {
            message: message.to_string(),
            start,
            end,
        };
        self.errors.push(error);
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
            writeln!(f, "Warning: {}", warning.message)?;
            writeln!(f, " --> {}", warning.start)?;
        }
        for error in self.errors.iter() {
            writeln!(f, "Error: {}", error.message)?;
            writeln!(f, " --> {}", error.start)?;
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

/// How severe is error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Warning,
    Error,
}
