use once_cell::sync::Lazy;
use std::{fmt::Display, str::FromStr};
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

static DICT: [&'static str; 34] = [
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Add,
    Sub,
    Not,
}

impl TryFrom<Punctuation> for UnaryOp {
    type Error = ();

    fn try_from(value: Punctuation) -> Result<Self, Self::Error> {
        use UnaryOp::*;
        Ok(match value.0 {
            "+" => Add,
            "-" => Sub,
            "!" => Not,
            _ => return Err(()),
        })
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnaryOp::*;
        write!(
            f,
            "{}",
            match self {
                Add => "+",
                Sub => "-",
                Not => "!",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Rsh,
    Lsh,
    BinAnd,
    BinOr,
    BinXor,
    And,
    Or,
    Eq,
    Neq,
    More,
    Less,
    MoreEq,
    LessEq,
}

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

impl TryFrom<Punctuation> for BinaryOp {
    type Error = ();

    fn try_from(value: Punctuation) -> Result<Self, Self::Error> {
        use BinaryOp::*;
        Ok(match value.0 {
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "%" => Mod,
            ">>" => Rsh,
            "<<" => Lsh,
            "&" => BinAnd,
            "|" => BinOr,
            "^" => BinXor,
            "&&" => And,
            "||" => Or,
            "==" => Eq,
            "!=" => Neq,
            ">" => More,
            "<" => Less,
            ">=" => MoreEq,
            "<=" => LessEq,
            _ => return Err(()),
        })
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinaryOp::*;
        write!(
            f,
            "{}",
            match self {
                Add => "+",
                Sub => "-",
                Mul => "*",
                Div => "/",
                Mod => "%",
                Rsh => ">>",
                Lsh => "<<",
                BinAnd => "&",
                BinOr => "|",
                BinXor => "^",
                And => "&&",
                Or => "||",
                Eq => "==",
                Neq => "!=",
                More => ">",
                Less => "<",
                MoreEq => ">=",
                LessEq => "<=",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

impl TryFrom<Punctuation> for AssignOp {
    type Error = ();

    fn try_from(value: Punctuation) -> Result<Self, Self::Error> {
        use AssignOp::*;
        Ok(match value.0 {
            "=" => Assign,
            "+=" => AddAssign,
            "-=" => SubAssign,
            "*=" => MulAssign,
            "/=" => DivAssign,
            _ => return Err(()),
        })
    }
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AssignOp::*;
        write!(
            f,
            "{}",
            match self {
                Assign => "=",
                AddAssign => "+=",
                SubAssign => "-=",
                MulAssign => "*=",
                DivAssign => "/=",
            }
        )
    }
}
