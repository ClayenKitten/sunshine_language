use crate::{
    ast::Identifier,
    lexer::{
        keyword::Keyword,
        operator::{BinaryOp, UnaryOp},
        punctuation::Punctuation,
        Lexer, LexerError, Token,
    },
    parser::ParserError,
};

use super::operator::AssignOp;

/// Utility methods over basic Lexer's iteration.
impl Lexer {
    /// Check if the following token is provided punctuation without advancing.
    pub fn peek_punctuation(&mut self, punc: &'static str) -> bool {
        let Ok(token) = self.peek() else { return false; };
        token == Token::Punc(Punctuation(punc))
    }

    /// Checks if next token is provided punctuation and consumes it if so.
    ///
    /// # Returns
    ///
    /// Returns `true` if provided punctuation matches.
    pub fn consume_punctuation(&mut self, punc: &'static str) -> Result<bool, LexerError> {
        if self.peek()? == Token::Punc(Punctuation(punc)) {
            self.discard();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if next token is provided keyword and consumes it if so.
    pub fn consume_keyword(&mut self, kw: Keyword) -> Result<bool, LexerError> {
        if self.peek()? == Token::Kw(kw) {
            self.discard();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if next token is identifier and consumes it if so.
    pub fn consume_identifier(&mut self) -> Result<Option<Identifier>, LexerError> {
        let Token::Ident(ident) = self.peek()? else { return Ok(None); };
        self.discard();
        Ok(Some(Identifier(ident)))
    }

    /// Checks if next token is unary operator and consumes it if so.
    pub fn consume_unary_operator(&mut self) -> Result<Option<UnaryOp>, LexerError> {
        let Token::Punc(punc) = self.peek()? else { return Ok(None); };
        match UnaryOp::try_from(punc) {
            Ok(op) => {
                self.discard();
                Ok(Some(op))
            }
            Err(_) => Ok(None),
        }
    }

    /// Checks if next token is binary operator and consumes it if so.
    pub fn consume_binary_operator(&mut self) -> Result<Option<BinaryOp>, LexerError> {
        let Token::Punc(punc) = self.peek()? else { return Ok(None); };
        let Ok(op) = BinaryOp::try_from(punc) else { return Ok(None); };
        self.discard();
        Ok(Some(op))
    }

    /// Checks if next token is assignment operator and consumes it if so.
    pub fn consume_assignment_operator(&mut self) -> Result<Option<AssignOp>, LexerError> {
        let Token::Punc(punc) = self.peek()? else { return Ok(None); };
        let Ok(op) = AssignOp::try_from(punc) else { return Ok(None); };
        self.discard();
        Ok(Some(op))
    }

    /// Check if next token is provided punctuation or error otherwise.
    pub fn expect_punctuation(&mut self, expected: &'static str) -> Result<(), ParserError> {
        let start = self.location;
        let found = self.next()?;
        if found == Token::Punc(Punctuation(expected)) {
            Ok(())
        } else {
            self.context.error_reporter.lock().unwrap().error(
                format!("Expected punctuation `{expected}`, found {found:?}"),
                self.source(),
                start,
                self.location,
            );
            Err(ParserError::UnexpectedToken(found))
        }
    }

    /// Check if next token is provided punctuation or error otherwise.
    pub fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), ParserError> {
        let start = self.location;
        let found = self.next()?;
        if found == Token::Kw(keyword) {
            Ok(())
        } else {
            self.context.error_reporter.lock().unwrap().error(
                format!("Expected keyword `{keyword}`, found {found:?}"),
                self.source(),
                start,
                self.location,
            );
            Err(ParserError::UnexpectedToken(found))
        }
    }

    /// Check if next token is identifier or error otherwise.
    pub fn expect_identifier(&mut self) -> Result<Identifier, ParserError> {
        let start = self.location;
        let found = self.next()?;
        if let Token::Ident(ident) = found {
            Ok(Identifier(ident))
        } else {
            self.context.error_reporter.lock().unwrap().error(
                format!("Expected identifier, found {found:?}"),
                self.source(),
                start,
                self.location,
            );
            Err(ParserError::UnexpectedToken(found))
        }
    }
}
