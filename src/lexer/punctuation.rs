use std::str::FromStr;
use thiserror::Error;

use super::{TokenStream, Token, LexerError};

pub fn parse(lexer: &mut TokenStream) -> Result<Token, LexerError> {
    const MAX_LENGTH: usize = 3;

    let buffer = fill_buf_with_punc(lexer, MAX_LENGTH);    
    
    let first_length = usize::min(MAX_LENGTH, buffer.len());
    for length in (0..=first_length).rev() {
        let slice = buffer.get(0..length).unwrap();
        if let Ok(punc) = Punctuation::from_str(slice) {
            lexer.stream.discard(buffer.len() - slice.len());
            let token = match Operator::try_from(punc) {
                Ok(op) => Token::Operator(op),
                Err(_) => Token::Punctuation(punc),
            };
            return Ok(token);
        }
    }
    Err(LexerError::UnknownPunctuation(NotPunctuation(buffer)))
}

fn fill_buf_with_punc(lexer: &mut TokenStream, max_length: usize) -> String {
    let mut buffer = String::with_capacity(max_length);
    loop {
        match lexer.stream.next() {
            Some(ch) if ch.is_ascii_punctuation() => buffer.push(ch),
            Some(_) => { lexer.stream.discard(1); break; },
            _ => break,
        }

        if buffer.len() >= max_length {
            break;
        }
    }
    buffer
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Punctuation(pub &'static str);

impl Punctuation {
    const DICT: [&'static str; 29] = [
        ";", ":", "{", "}", "(", ")", "[", "]", ",", "->",
        "=", "+", "+=", "-", "-=", "*", "*=", "/", "/=", "%", "%=",
        "==", "!=", ">", "<", ">=", "<=", "&&", "||"
    ];

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
