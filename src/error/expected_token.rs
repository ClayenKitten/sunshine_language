use std::fmt::Display;

use crate::lexer::{keyword::Keyword, punctuation::Punctuation};

/// Type of the token that was expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedToken {
    Identifier,
    Keyword(Keyword),
    Punctuation(Punctuation),
}

impl From<Keyword> for ExpectedToken {
    fn from(val: Keyword) -> Self {
        ExpectedToken::Keyword(val)
    }
}

impl From<Punctuation> for ExpectedToken {
    fn from(val: Punctuation) -> Self {
        ExpectedToken::Punctuation(val)
    }
}

impl Display for ExpectedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedToken::Identifier => write!(f, "identifier"),
            ExpectedToken::Keyword(kw) => write!(f, "keyword `{kw}`"),
            ExpectedToken::Punctuation(punc) => write!(f, "`{punc}`"),
        }
    }
}
