use crate::{
    ast::statement::LetStatement,
    parser::{FileParser, ParserError},
};

/// [Statement]'s parsing.
///
/// [Statement]: crate::ast::statement::Statement
impl<'s> FileParser<'s> {
    /// Parse let statement. [let] keyword is expected to be consumed beforehand.
    ///
    /// [let]: crate::lexer::keyword::Keyword::Let
    pub fn parse_let(&mut self) -> Result<LetStatement, ParserError> {
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
