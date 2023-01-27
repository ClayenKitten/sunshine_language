use once_cell::sync::Lazy;
use std::str::FromStr;
use thiserror::Error;

use super::{Lexer, LexerError, Token};

impl Lexer {
    /// Try to parse punctuation or operator from input stream.
    ///
    /// Longest sequence of chars that represents punctuation is considered a token. So, `->` is returned rather than `-`.
    pub(super) fn read_punctuation(&mut self) -> Result<Token, LexerError> {
        let mut buffer = String::with_capacity(*MAX_PUNCTUATION_LENGTH);
        let mut result = None;
        for i in 0..*MAX_PUNCTUATION_LENGTH {
            let Some(ch) = self.input.peek_nth(i) else { break };
            if !ch.is_ascii_punctuation() {
                break;
            }
            buffer.push(ch);

            result = Punctuation::from_str(&buffer).ok().or(result);
        }

        result
            .map(|punc| {
                self.input.nth(punc.0.len() - 1);
                Token::Punc(punc)
            })
            .ok_or(LexerError::UnknownPunctuation(NotPunctuation(buffer)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Punctuation(pub &'static str);

static DICT: [&str; 34] = [
    ";", ":", "{", "}", "(", ")", "[", "]", ",", "->", "+", "-", "!", "*", "/", "%", ">>", "<<",
    "&", "^", "|", "&&", "||", "==", "!=", ">", "<", ">=", "<=", "=", "+=", "-=", "*=", "/=",
];

static MAX_PUNCTUATION_LENGTH: Lazy<usize> =
    Lazy::new(|| DICT.iter().map(|punc| punc.len()).max().unwrap_or_default());

impl Punctuation {
    pub fn new(s: &'static str) -> Self {
        if DICT.contains(&s) {
            Self(s)
        } else {
            panic!("Invalid punctuation `{s}`");
        }
    }
}

impl FromStr for Punctuation {
    type Err = NotPunctuation;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = DICT.iter().find(|c| *c == &s) {
            Ok(Punctuation(s))
        } else {
            Err(NotPunctuation(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("provided string is not punctuation")]
pub struct NotPunctuation(String);
