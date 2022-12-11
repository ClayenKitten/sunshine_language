use crate::{
    ast::expressions::{Expression, For, If, While},
    lexer::keyword::Keyword,
};

use super::{Parser, ParserError};

impl<'s> Parser<'s> {
    /// Parse if loop. Keyword `if` is expected to be consumed beforehand.
    pub fn parse_if(&mut self) -> Result<If, ParserError> {
        let condition = Box::new(Expression::parse(&mut self.lexer)?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;

        let else_body = if self.lexer.consume_keyword(Keyword::Else)? {
            self.lexer.expect_punctuation("{")?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(If {
            condition,
            body,
            else_body,
        })
    }

    /// Parse while loop. Keyword `while` is expected to be consumed beforehand.
    pub fn parse_while(&mut self) -> Result<While, ParserError> {
        let condition = Box::new(Expression::parse(&mut self.lexer)?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;
        Ok(While { condition, body })
    }

    /// Parse for loop. Keyword `for` is expected to be consumed beforehand.
    pub fn parse_for(&mut self) -> Result<For, ParserError> {
        let var = self.lexer.expect_identifier()?;
        self.lexer.expect_keyword(Keyword::In)?;
        let expr = Box::new(Expression::parse(&mut self.lexer)?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;
        Ok(For { var, expr, body })
    }
}
