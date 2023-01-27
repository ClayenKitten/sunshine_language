//! Error reporting.

use std::{fmt::Display, sync::{Arc, Mutex}};

use crate::{input_stream::Location, source::{SourceMap, SourceId}};

/// Interface to report errors conveniently.
#[derive(Debug, Clone)]
pub struct ErrorReporter {
    source_map: Arc<Mutex<SourceMap>>,
    /// Valid code that looks suspicious to compiler.
    warnings: Vec<Error>,
    /// Invalid code that doesn't allow compilation to proceed.
    errors: Vec<Error>,
}

impl ErrorReporter {
    /// Create new ErrorReporter.
    pub fn new(source_map: Arc<Mutex<SourceMap>>) -> Self {
        Self {
            source_map,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Build error.
    pub fn error(&mut self, message: impl ToString, file: Option<SourceId>, start: Location, end: Location) {
        let error = Error {
            message: message.to_string(),
            file,
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

impl Display for ErrorReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for warning in self.warnings.iter() {
            writeln!(f, "Warning: {}", warning.message)?;
            writeln!(f, " --> {}", warning.start)?;
        }
        for error in self.errors.iter() {
            writeln!(f, "Error: {}", error.message)?;
            if let Some(file) = error.file {
                writeln!(
                    f, " --> {}:{}",
                    self.source_map
                        .lock()
                        .unwrap()
                        .get_path(file)
                        .to_string_lossy(),
                    error.start
                )?
            } else {
                writeln!(f, " --> {}", error.start)?
            }
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
    file: Option<SourceId>,
    start: Location,
    end: Location,
}

/// How severe is error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Warning,
    Error,
}
