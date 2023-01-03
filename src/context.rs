use std::sync::Mutex;

use crate::error::ErrorReporter;

#[derive(Debug)]
pub struct Context {
    pub metadata: Metadata,
    pub error_reporter: Mutex<ErrorReporter>,
}

impl Context {
    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            metadata: Metadata { crate_name: String::from("_TEST") },
            error_reporter: Mutex::new(ErrorReporter::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub crate_name: String,
}
