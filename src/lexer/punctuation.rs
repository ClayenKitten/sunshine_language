use once_cell::sync::Lazy;
use std::str::FromStr;
use thiserror::Error;

use super::{Lexer, LexerError, Token};
use crate::util::count;

impl Lexer {
    /// Try to parse punctuation or operator from input stream.
    ///
    /// Longest sequence of chars that represents punctuation is considered a token. So, `->` is returned rather than `-`.
    pub(super) fn read_punctuation(&mut self) -> Result<Token, LexerError> {
        let mut buffer = String::with_capacity(*MAX_PUNC_LENGTH);
        let mut result = None;
        for i in 0..*MAX_PUNC_LENGTH {
            let Some(ch) = self.input.peek_nth(i) else { break };
            if !ch.is_ascii_punctuation() {
                break;
            }
            buffer.push(ch);

            result = Punctuation::from_str(&buffer).ok().or(result);
        }

        result
            .map(|punc| {
                self.input.nth(punc.as_str().len() - 1);
                Token::Punc(punc)
            })
            .ok_or(LexerError::UnknownPunctuation(NotPunctuation(buffer)))
    }
}

macro_rules! punc {
    ($($identifier:ident = $symbol:literal,)*) => {
        const PUNC_NUMBER: usize = count!($($symbol)*);
        static DICT: [&str; PUNC_NUMBER] = [$($symbol,)*];
        static MAX_PUNC_LENGTH: Lazy<usize> =
            Lazy::new(|| DICT.iter().map(|punc| punc.len()).max().unwrap_or_default());

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Punctuation {
            $($identifier,)*
        }

        impl Punctuation {
            pub fn new(s: &'static str) -> Self {
                match Self::from_str(s) {
                    Ok(punc) => punc,
                    Err(_) => panic!("Invalid punctuation `{s}`"),
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$identifier => $symbol,)*
                }
            }
        }

        impl std::fmt::Display for Punctuation {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl FromStr for Punctuation {
            type Err = NotPunctuation;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    $($symbol => Self::$identifier,)*
                    _ => return Err(NotPunctuation(s.to_string())),
                })
            }
        }
    };
}

punc![
    Semicolon = ";",
    Colon = ":",
    LBrace = "{",
    RBrace = "}",
    LParent = "(",
    RParent = ")",
    LBracket = "[",
    RBracket = "]",
    Comma = ",",
    Arrow = "->",
    Plus = "+",
    Minus = "-",
    Bang = "!",
    Mul = "*",
    Div = "/",
    Rem = "%",
    Rsh = ">>",
    Lsh = "<<",
    BinAnd = "&",
    BinXor = "^",
    BinOr = "|",
    And = "&&",
    Or = "||",
    Equal = "==",
    NotEqual = "!=",
    More = ">",
    Less = "<",
    MoreEqual = ">=",
    LessEqual = "<=",
    Assign = "=",
    AssignPlus = "+=",
    AssignMinus = "-=",
    AssignMul = "*=",
    AssignDiv = "/=",
];

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("provided string is not punctuation")]
pub struct NotPunctuation(String);
