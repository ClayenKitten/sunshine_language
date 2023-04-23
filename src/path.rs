use crate::identifier::IdentifierParseError;
use thiserror::Error;

mod absolute;
mod relative;

pub use absolute::AbsolutePath;
pub use relative::{RelativePath, RelativePathStart};

#[derive(Debug, PartialEq, Eq, Error)]
pub enum PathParsingError {
    #[error("expected identifier")]
    ExpectedIdentifier,
    #[error("invalid identifier, {0}")]
    InvalidIdentifier(#[from] IdentifierParseError),
}
