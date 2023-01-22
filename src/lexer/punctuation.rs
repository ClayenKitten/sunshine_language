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
                Token::Punctuation(punc)
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

macro_rules! define_operator {
    (
        $(#[doc = $doc:expr])?
        enum $name:ident {
            $($field:ident = $value:literal,)*
        }
    ) => {
        $(#[doc = $doc])?
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($field,)*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        $($name::$field => $value,)*
                    }
                )
            }
        }

        impl TryFrom<Punctuation> for $name {
            type Error = ();

            fn try_from(value: Punctuation) -> Result<Self, Self::Error> {
                Ok(match value.0 {
                    $($value => $name::$field,)*
                    _ => return Err(()),
                })
            }
        }
    }
}

define_operator!(
    /// An operator with one operand.
    enum UnaryOp {
        Add = "+",
        Sub = "-",
        Not = "!",
    }
);

define_operator!(
    /// An operator with two operands.
    enum BinaryOp {
        Add = "+",
        Sub = "-",
        Mul = "*",
        Div = "/",
        Mod = "%",
        Rsh = ">>",
        Lsh = "<<",
        BinAnd = "&",
        BinOr = "|",
        BinXor = "^",
        And = "&&",
        Or = "||",
        Eq = "==",
        Neq = "!=",
        More = ">",
        Less = "<",
        MoreEq = ">=",
        LessEq = "<=",
    }
);

impl BinaryOp {
    pub fn priority(&self) -> usize {
        use BinaryOp::*;
        match self {
            Mul | Div | Mod => 128,
            Add | Sub => 96,
            Rsh | Lsh => 64,
            BinAnd => 52,
            BinXor => 51,
            BinOr => 50,
            And => 31,
            Or => 30,
            Eq | Neq | More | Less | MoreEq | LessEq => 16,
        }
    }
}

define_operator!(
    /// An operator with two operands: assignee and value.
    enum AssignOp {
        Assign = "=",
        AddAssign = "+=",
        SubAssign = "-=",
        MulAssign = "*=",
        DivAssign = "/=",
    }
);
