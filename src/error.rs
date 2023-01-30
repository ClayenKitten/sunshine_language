//! Error reporting.

pub mod library;
mod types;

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    input_stream::Location,
    source::{SourceId, SourceMap},
};

pub use self::types::*;

/// Interface to report errors conveniently.
#[derive(Debug)]
pub struct ErrorReporter {
    source_map: Arc<Mutex<SourceMap>>,
    errors: Mutex<Vec<Box<dyn ReportableError>>>,
}

impl ErrorReporter {
    /// Create new ErrorReporter.
    pub fn new(source_map: Arc<Mutex<SourceMap>>) -> Self {
        Self {
            source_map,
            errors: Mutex::new(Vec::new()),
        }
    }

    pub fn report(&self, error: impl ReportableError + 'static) {
        self.errors.lock().unwrap().push(Box::new(error));
    }

    /// Check if any fatal error occurred.
    pub fn compilation_failed(&self) -> bool {
        !self.errors.lock().unwrap().is_empty()
    }

    /// Calculates number of warnings and errors.
    fn calc_number(&self) -> (usize, usize) {
        self.errors
            .lock()
            .unwrap()
            .iter()
            .fold((0, 0), |(w, e), err| match err.severity() {
                Severity::Warn => (w + 1, e),
                Severity::Deny => (w, e + 1),
            })
    }
}

impl Display for ErrorReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in self.errors.lock().unwrap().iter() {
            match error.severity() {
                Severity::Warn => writeln!(f, "Warning: {}", error)?,
                Severity::Deny => writeln!(f, "Error: {}", error)?,
            }
            match error.span().source {
                Some(file) => writeln!(
                    f,
                    " --> {}:{}",
                    self.source_map
                        .lock()
                        .unwrap()
                        .get_path(file)
                        .to_string_lossy(),
                    error.span().start
                )?,
                None => writeln!(f, " --> {}", error.span().start)?,
            }
            writeln!(f)?;
        }
        let (warnings, error) = self.calc_number();
        writeln!(f, "{warnings} warning(s), {error} error(s)",)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
    source: Option<SourceId>,
    start: Location,
    end: Location,
}
