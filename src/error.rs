//! Error reporting.

mod error_reporter;
mod expected_token;
pub mod library;
mod report_provider;

pub use error_reporter::*;
pub use expected_token::*;
pub use report_provider::*;
use thiserror::Error;

use std::error::Error;

use crate::{
    lexer::{LexerError, Token},
    source::SourceError,
    util::Span,
};

/// Error that may be reported.
pub trait ReportableError: Error {
    fn severity(&self) -> Severity;
    fn span(&self) -> Span;
}

/// Fatal error occured during compilation.
#[derive(Debug, Clone, Copy, Error)]
#[error("error occured during compilation")]
pub struct CompilerError;

impl From<LexerError> for CompilerError {
    fn from(_value: LexerError) -> Self {
        CompilerError
    }
}

impl From<SourceError> for CompilerError {
    fn from(_value: SourceError) -> Self {
        CompilerError
    }
}

/// How severe is the error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// User attention requested.
    Warn,
    /// Compilation failed.
    Deny,
}

impl Token {
    fn pretty_print(&self) -> String {
        match self {
            Token::Punc(punc) => format!("`{punc}`"),
            Token::Num(num) => format!("number `{num}`"),
            Token::Str(s) => format!("\"{s}\""),
            Token::Kw(kw) => format!("keyword `{kw}`"),
            Token::Ident(ident) => format!("`{ident}`"),
            Token::Eof => todo!(),
        }
    }
}
