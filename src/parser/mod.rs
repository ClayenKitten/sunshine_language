use thiserror::Error;

use crate::lexer::{TokenStream, LexerError, Token, TokenKind, keyword::Keyword, punctuation::Punctuation};

use self::{expressions::*, item::Item, statement::Statement};

pub mod expressions;
mod item;
mod statement;

#[derive(Debug)]
pub struct Ast(Vec<Item>);

impl Ast {
    /// Parse top level of program (file).
    pub fn parse(lexer: &mut TokenStream) -> Result<Ast, ParserError> {    
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
    pub fn is_matching_closing_delimiter(&self, s: &str) -> bool {
        matches!(
            (self, s),
            (Delimiter::Parenthesis, ")") |
            (Delimiter::Brace, "}") |
            (Delimiter::Bracket, "]")
        )
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

impl<'a> TokenStream<'a> {
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

    fn expect_punctuation(&mut self, punc: &'static [&'static str]) -> Result<(), UnexpectedTokenError> {
        match self.next()? {
            Token::Punctuation(got) if punc.contains(&got.0) => Ok(()),
            _ => Err(UnexpectedTokenError::TokenMismatch),
        }
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

    fn next_expected_kind(&mut self, expected: TokenKind) -> Result<Token, UnexpectedTokenError> {
        match self.next()? {
            token if expected == (&token).into() => Ok(token),
            token => Err(UnexpectedTokenError::TypeMismatch { expected, got: token.into() }),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnexpectedTokenError {
    #[error("unexpected token")]
    UnexpectedToken(Token),
    #[error("token mismatch")]
    TokenMismatch,
    #[error("token type mismatch")]
    TypeMismatch { expected: TokenKind, got: TokenKind },
    #[error("{0}")]
    LexerError(#[from] LexerError),
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParserError {
    #[error("Invalid top-level token.")]
    InvalidTopLevel,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Lexer error occured: {0}.")]
    LexerError(#[from] LexerError),
}
