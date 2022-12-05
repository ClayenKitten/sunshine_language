use thiserror::Error;

use crate::lexer::{Lexer, LexerError, Token, keyword::Keyword, punctuation::Punctuation};

use self::{expressions::*, item::Item, statement::Statement};

pub mod expressions;
mod item;
mod statement;

#[derive(Debug)]
pub struct Ast(Vec<Item>);

impl Ast {
    /// Parse top level of program (file).
    pub fn parse(lexer: &mut Lexer) -> Result<Ast, ParserError> {    
        let mut buffer = Vec::new();
        while !lexer.is_eof() {
            buffer.push(Item::parse(lexer)?);
        }
        Ok(Ast(buffer))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// ( ... )
    Parenthesis,
    /// { ... }
    Brace,
    /// [ ... ]
    Bracket,
}

impl Delimiter {
    /// Check if provided `str` contains a matching closing delimiter.
    pub fn is_closing(&self, s: &str) -> bool {
        matches!(
            (self, s),
            (Delimiter::Parenthesis, ")") |
            (Delimiter::Brace, "}") |
            (Delimiter::Bracket, "]")
        )
    }
}

impl TryFrom<char> for Delimiter {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '(' | ')' => Ok(Delimiter::Parenthesis),
            '{' | '}' => Ok(Delimiter::Brace),
            '[' | ']' => Ok(Delimiter::Bracket),
            _ => Err(()),
        }
    }
}

impl TryFrom<Punctuation> for Delimiter {
    type Error = ();

    fn try_from(value: Punctuation) -> Result<Self, Self::Error> {
        match value.0 {            
            "(" | ")" => Ok(Delimiter::Parenthesis),
            "{" | "}" => Ok(Delimiter::Brace),
            "[" | "]" => Ok(Delimiter::Bracket),
            _ => Err(()),
        }
    }
}

impl<'a> Lexer<'a> {
    fn expect(&mut self, criteria: impl Fn(&Token) -> bool) -> Result<(), UnexpectedTokenError> {
        let token = self.next()?;
        if criteria(&token) {
            Ok(())
        } else {
            Err(UnexpectedTokenError::UnexpectedToken(token))
        }
    }

    fn extract<T>(&mut self, extractor: impl Fn(&Token) -> Option<T>) -> Result<T, UnexpectedTokenError> {
        let token = self.next()?;
        if let Some(val) = extractor(&token) {
            Ok(val)
        } else {
            Err(UnexpectedTokenError::UnexpectedToken(token))
        }
    }

    /// Checks if next token is provided punctuation and consumes it if so.
    /// 
    /// # Returns
    /// 
    /// Returns `true` if provided punctuation matches.
    fn consume_punctuation(&mut self, punc: &'static str) -> Result<bool, ParserError> {
        if Token::Punctuation(Punctuation(punc)) == self.peek()? {
            let _ = self.next();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Fetch next token and check if it is one of listed punctuation.
    /// 
    /// # Returns
    /// 
    /// Returned str is guaranteed to be one of provided in `punc`.
    fn expect_punctuation(&mut self, expected: impl IntoIterator<Item = &'static str>) -> Result<&'static str, ParserError> {
        let Token::Punctuation(Punctuation(punc)) = self.next_some()? else {
            return Err(UnexpectedTokenError::TokenMismatch.into());
        };

        expected.into_iter()
            .find(|x| *x == punc)
            .ok_or_else(|| UnexpectedTokenError::TokenMismatch.into())
    }

    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), UnexpectedTokenError> {
        match self.next()? {
            Token::Keyword(got) if got == keyword => Ok(()),
            _ => Err(UnexpectedTokenError::TokenMismatch),
        }
    }

    /// Returns error if EOF achieved.
    fn next_some(&mut self) -> Result<Token, ParserError> {
        match self.next()? {
            Token::Eof => Err(ParserError::UnexpectedEof),
            token => Ok(token),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnexpectedTokenError {
    #[error("unexpected token: ")]
    UnexpectedToken(Token),
    #[error("token mismatch")]
    TokenMismatch,
    #[error("{0}")]
    LexerError(#[from] LexerError),
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParserError {
    #[error(transparent)]
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Lexer error occured: {0}.")]
    LexerError(#[from] LexerError),
}
