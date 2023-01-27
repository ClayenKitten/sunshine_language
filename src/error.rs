//! Error reporting.

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    input_stream::Location,
    source::{SourceId, SourceMap},
};

/// Interface to report errors conveniently.
#[derive(Debug, Clone)]
pub struct ErrorReporter {
    source_map: Arc<Mutex<SourceMap>>,
    errors: Vec<(Severity, Error)>,
}

impl ErrorReporter {
    /// Create new ErrorReporter.
    pub fn new(source_map: Arc<Mutex<SourceMap>>) -> Self {
        Self {
            source_map,
            errors: Vec::new(),
        }
    }

    /// Build warning.
    pub fn warn(
        &mut self,
        message: impl ToString,
        file: Option<SourceId>,
        start: Location,
        end: Location,
    ) {
        let error = Error {
            message: message.to_string(),
            file,
            start,
            end,
        };
        self.errors.push((Severity::Warning, error));
    }

    /// Build error.
    pub fn error(
        &mut self,
        message: impl ToString,
        file: Option<SourceId>,
        start: Location,
        end: Location,
    ) {
        let error = Error {
            message: message.to_string(),
            file,
            start,
            end,
        };
        self.errors.push((Severity::Error, error));
    }

    /// Check if any fatal error occurred.
    pub fn compilation_failed(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Calculates number of warnings and errors.
    fn calc_number(&self) -> (usize, usize) {
        self.errors
            .iter()
            .fold((0, 0), |(w, e), (severity, _)| match severity {
                Severity::Warning => (w + 1, e),
                Severity::Error => (w, e + 1),
            })
    }
}

impl Display for ErrorReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (severity, error) in self.errors.iter() {
            match severity {
                Severity::Warning => writeln!(f, "Warning: {}", error.message)?,
                Severity::Error => writeln!(f, "Error: {}", error.message)?,
            }
            match error.file {
                Some(file) => writeln!(
                    f,
                    " --> {}:{}",
                    self.source_map
                        .lock()
                        .unwrap()
                        .get_path(file)
                        .to_string_lossy(),
                    error.start
                )?,
                None => writeln!(f, " --> {}", error.start)?,
            }
        }
        let (warnings, error) = self.calc_number();
        writeln!(f, "{warnings} warning(s), {error} error(s)",)?;
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
