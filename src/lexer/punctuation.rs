use std::{str::FromStr, collections::HashMap};
use once_cell::sync::Lazy;
use thiserror::Error;

use super::{Lexer, Token, LexerError};

impl<'a> Lexer<'a> {
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
                Token::Punctuation(punc)
            })
            .ok_or(LexerError::UnknownPunctuation(NotPunctuation(buffer)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Punctuation(pub &'static str);

/// A list of properties of punctuation token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PuncProps {
    /// Does that punctuation represent prefixed unary operator
    pub is_unary_op: bool,
    /// Priority of binary operator
    pub binary_priority: Option<u8>,
    /// Assignment operator may only appear once in an expression
    pub is_assign: bool,
}

static DICT: Lazy<HashMap<&'static str, PuncProps>> = Lazy::new(|| {
    let punc = [";", ":", "{", "}", "(", ")", "[", "]", ",", "->"];

    let unary = ["+", "-", "!"];
    let assign = ["=", "+=", "-=", "*=", "/="];
    let binary = [
        ("*", 128),
        ("/", 128),
        ("%", 128),
        
        ("+",  96),
        ("-",  96),
        
        (">>", 64),
        ("<<", 64),
        
        ("&", 50),
        ("^", 49),
        ("|", 48),
        
        ("&&", 32),
        ("||", 32),
        
        ("==", 16),
        ("!=", 16),
        (">",  16),
        ("<",  16),
        (">=", 16),
        ("<=", 16),
    ];
    
    punc.into_iter()
        .chain(unary)
        .chain(assign)
        .chain(binary.map(|b| b.0))
        .map(|s| {
            (s, PuncProps {
                is_unary_op: unary.contains(&s),
                binary_priority: binary.into_iter()
                    .find_map(|bin| (bin.0 == s).then_some(bin.1)),
                is_assign: assign.contains(&s)
            })
        })
        .collect()
});

static MAX_PUNCTUATION_LENGTH: Lazy<usize> = Lazy::new(|| {
    DICT.keys()
        .map(|punc| punc.len())
        .max()
        .unwrap_or_default()
});

impl Punctuation {
    pub fn new(s: &'static str) -> Self {
        if DICT.contains_key(s) {
            Self(s)
        } else {
            panic!("Invalid punctuation `{s}`");
        }
    }

    pub fn is_operator(&self) -> bool {
        self.is_unary_operator() || self.is_binary_operator()
    }

    pub fn is_unary_operator(&self) -> bool {
        DICT.get(self.0)
            .map(|prop| prop.is_unary_op)
            .unwrap_or_default()
    }

    pub fn is_binary_operator(&self) -> bool {
        DICT.get(self.0)
            .map(|prop| prop.binary_priority.is_some())
            .unwrap_or_default()
    }

    pub fn is_assignment_operator(&self) -> bool {
        DICT.get(self.0)
            .map(|prop| prop.is_assign)
            .unwrap_or_default()
    }

    pub fn binary_priority(&self) -> Option<u8> {
        DICT.get(self.0)
            .and_then(|prop| prop.binary_priority)
    }
}

impl FromStr for Punctuation {
    type Err = NotPunctuation;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DICT.get_key_value(s)
            .map(|(key, _)| Punctuation(key))
            .ok_or_else(|| NotPunctuation(s.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("provided string is not punctuation")]
pub struct NotPunctuation(String);
