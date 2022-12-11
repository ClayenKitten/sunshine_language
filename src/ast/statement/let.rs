use crate::{ast::{expressions::{Expression, Identifier}}, lexer::{Lexer, keyword::Keyword}, parser::ParserError};

/// let VAR: TYPE = VALUE;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Identifier,
    pub type_: Option<Identifier>,
    pub value: Option<Box<Expression>>,
}

impl LetStatement {
    pub fn parse(lexer: &mut Lexer) -> Result<LetStatement, ParserError> {
        lexer.expect_keyword(Keyword::Let)?;
        let name = lexer.expect_identifier()?;
        let mut statement = LetStatement {
            name,
            type_: None,
            value: None,
        };
        if lexer.consume_punctuation(":")? {
            statement.type_ = Some(lexer.expect_identifier()?);
        }
        if lexer.consume_punctuation("=")? {
            statement.value = Some(Box::new(Expression::parse(lexer)?));
        }
        lexer.expect_punctuation(";")?;
        Ok(statement)
    }
}