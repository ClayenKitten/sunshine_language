use crate::{ast::statement::LetStatement, lexer::keyword::Keyword};

use super::{Parser, ParserError};

impl<'s> Parser<'s> {
    /// Parse let statement. `let` keyword is expected to be consumed beforehand.
    pub fn parse_let(&mut self) -> Result<LetStatement, ParserError> {
        self.lexer.expect_keyword(Keyword::Let)?;
        let name = self.lexer.expect_identifier()?;
        let mut statement = LetStatement {
            name,
            type_: None,
            value: None,
        };
        if self.lexer.consume_punctuation(":")? {
            statement.type_ = Some(self.lexer.expect_identifier()?);
        }
        if self.lexer.consume_punctuation("=")? {
            statement.value = Some(Box::new(self.parse_expr()?));
        }
        self.lexer.expect_punctuation(";")?;
        Ok(statement)
    }
}
