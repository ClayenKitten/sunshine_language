//! Compiler context.

use std::{path::PathBuf, sync::Mutex};

use clap::ValueEnum;

use crate::{
    ast::Identifier,
    error::ErrorReporter,
    source::{SourceError, SourceMap},
};

#[derive(Debug)]
pub struct Context {
    pub metadata: Metadata,
    pub source: Mutex<SourceMap>,
    pub error_reporter: Mutex<ErrorReporter>,
}

impl Context {
    pub fn new(main: PathBuf, metadata: Metadata) -> Result<Context, SourceError> {
        Ok(Context {
            metadata,
            source: Mutex::new(SourceMap::new(main)?),
            error_reporter: Mutex::new(ErrorReporter::new()),
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            metadata: Metadata {
                crate_name: Identifier(String::from("_TEST")),
                emit_type: Emit::default(),
            },
            source: Mutex::new(SourceMap::new_test().unwrap()),
            error_reporter: Mutex::new(ErrorReporter::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub crate_name: Identifier,
    pub emit_type: Emit,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emit {
    Ast,
    LlvmIr,
    #[default]
    Binary,
}
