//! Compiler context.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clap::ValueEnum;

use crate::{
    ast::Identifier,
    error::ErrorReporter,
    source::{SourceError, SourceMap},
};

/// Context of the compilation.
///
/// That structure is cheap to clone as it only contains [`Arc`]s.
#[derive(Debug, Clone)]
pub struct Context {
    pub metadata: Arc<Metadata>,
    pub source: Arc<Mutex<SourceMap>>,
    pub error_reporter: Arc<Mutex<ErrorReporter>>,
}

impl Context {
    pub fn new(main: PathBuf, metadata: Metadata) -> Result<Context, SourceError> {
        Ok(Context {
            metadata: Arc::new(metadata),
            source: Arc::new(Mutex::new(SourceMap::new(main)?)),
            error_reporter: Arc::new(Mutex::new(ErrorReporter::new())),
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            metadata: Arc::new(Metadata {
                crate_name: Identifier(String::from("_TEST")),
                emit_type: Emit::default(),
            }),
            source: Arc::new(Mutex::new(SourceMap::new_test().unwrap())),
            error_reporter: Arc::new(Mutex::new(ErrorReporter::new())),
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
