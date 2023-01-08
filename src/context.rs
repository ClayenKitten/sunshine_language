use std::sync::Mutex;

use clap::ValueEnum;

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
            metadata: Metadata {
                crate_name: String::from("_TEST"),
                emit_type: Emit::default(),
            },
            error_reporter: Mutex::new(ErrorReporter::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub crate_name: String,
    pub emit_type: Emit,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emit {
    Ast,
    LlvmIr,
    #[default]
    Binary,
}
