use std::str::FromStr;
use thiserror::Error;

use super::{TokenStream, Token, LexerError};

impl<'a> TokenStream<'a> {
    /// Try to parse punctuation or operator from input stream.
    /// 
    /// Longest sequence of chars that represents punctuation is considered a token. So, `->` is returned rather than `-`.
    pub(super) fn read_punctuation(&mut self) -> Result<Token, LexerError> {
        let mut buffer = String::with_capacity(Punctuation::MAX_PUNCTUATION_LENGTH);
        let mut result = None;
        for i in 0..Punctuation::MAX_PUNCTUATION_LENGTH {
            let Some(ch) = self.stream.peek_nth(i) else { break };
            if !ch.is_ascii_punctuation() {
                break;
            }
            buffer.push(ch);

            result = Punctuation::from_str(&buffer).ok().or(result);
        }
        
        result
            .map(|punc| {
                self.stream.nth(punc.0.len() - 1);
                match Operator::try_from(punc) {
                    Ok(op) => Token::Operator(op),
                    Err(_) => Token::Punctuation(punc),
                }
            })
            .ok_or(LexerError::UnknownPunctuation(NotPunctuation(buffer)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Punctuation(pub &'static str);

impl Punctuation {
    const DICT: [&'static str; 29] = [
        ";", ":", "{", "}", "(", ")", "[", "]", ",", "->",
        "=", "+", "+=", "-", "-=", "*", "*=", "/", "/=", "%", "%=",
        "==", "!=", ">", "<", ">=", "<=", "&&", "||"
    ];

    const MAX_PUNCTUATION_LENGTH: usize = {
        let mut max_len = 0;
        let mut index = 0;
        while index < Self::DICT.len() {
            let len = Self::DICT[index].len();
            if len > max_len {
                max_len = len
            }
            index += 1;
        }
        max_len
    };

    pub fn new(s: &'static str) -> Self {
        if Self::DICT.contains(&s) {
            Self(s)
        } else {
            panic!("Invalid punctuation");
        }
    }
}

impl FromStr for Punctuation {
    type Err = NotPunctuation;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::DICT.iter().position(|i| *i == s) {
            Some(index) => Ok(Punctuation(Self::DICT[index])),
            None => Err(NotPunctuation(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Assign,

    Plus,
    PlusAssign,

    Minus,
    MinusAssign,

    Multiply,
    MultiplyAssign,

    Divide,
    DivideAssign,

    Modulo,
    ModuloAssign,

    Equal,
    NotEqual,
    More,
    Less,
    MoreOrEqual,
    LessOrEqual,

    And,
    Or,
    // TODO: Bitwise logic & shift operators
}

impl TryFrom<Punctuation> for Operator {
    type Error = NotAnOperator;

    fn try_from(value: Punctuation) -> Result<Self, Self::Error> {
        use Operator::*;

        Ok(match value.0 {
            "=" => Assign,
            "+" => Plus,
            "+=" => PlusAssign,
            "-" => Minus,
            "-=" => MinusAssign,
            "*" => Multiply,
            "*=" => MultiplyAssign,
            "/" => Divide,
            "/=" => DivideAssign,
            "%" => Modulo,
            "%=" => ModuloAssign,
            "==" => Equal,
            "!=" => NotEqual,
            ">" => More,
            "<" => Less,
            ">=" => MoreOrEqual,
            "<=" => LessOrEqual,
            "&&" => And,
            "||" => Or,
            s => return Err(NotAnOperator(s)),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("provided string is not punctuation")]
pub struct NotPunctuation(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("attempt to parse punctuation `{0}` as an operator failed")]
pub struct NotAnOperator(&'static str);
