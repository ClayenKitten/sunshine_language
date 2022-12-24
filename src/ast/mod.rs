use std::{fmt::Display, str::FromStr};
use thiserror::Error;

pub mod expression;
pub mod item;
pub mod statement;

/// Identifier is name of type, variable or function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Identifier {
    type Err = IdentifierParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {        
        if s.is_empty() {
            return Err(IdentifierParseError::Empty);
        }
        if s.starts_with(|ch: char| ch.is_ascii_digit()) {
            return Err(IdentifierParseError::StartsWithNumber);
        }
        if let Some(ch) = s.chars().find(|ch| !(ch.is_ascii_alphanumeric() || *ch == '_')) {
            return Err(IdentifierParseError::InvalidCharacter(ch));
        }
        
        Ok(Identifier(s.to_string()))
    }
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum IdentifierParseError {
    #[error("identifier shouldn't start with a number")]
    StartsWithNumber,
    #[error("identifier may only contain ascii alphanumeric and underscore characters, character `{0}` met")]
    InvalidCharacter(char),
    #[error("identifier can't be empty")]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Visibility {
    Public,
    #[default]
    Private,
}

#[cfg(test)]
mod test {
    #[test]
    fn visibility_ordering() {
        use super::Visibility::*;
        let expected = vec![Public, Public, Private, Private];
        let mut init = vec![Private, Public, Private, Public];
        init.sort();
        assert_eq!(expected, init);
    }
}
