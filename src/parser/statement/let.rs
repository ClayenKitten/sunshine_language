use crate::{parser::{ParserError, expressions::{Expression, Identifier}}, lexer::{TokenStream, keyword::Keyword}};

/// let VAR: TYPE = VALUE;
#[derive(Debug, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Identifier,
    pub type_: Option<Identifier>,
    pub value: Option<Box<Expression>>,
}

impl LetStatement {
    pub fn parse(lexer: &mut TokenStream) -> Result<LetStatement, ParserError> {
        lexer.expect_keyword(Keyword::Let)?;
        let mut statement = LetStatement {
            name: Identifier::parse(lexer)?,
            type_: None,
            value: None,
        };
        
        match lexer.expect_punctuation([":", ";"])? {
            ":" => statement.type_ = Some(Identifier::parse(lexer)?),
            ";" => return Ok(statement),
            _ => unreachable!(),
        }

        match lexer.expect_punctuation(["=", ";"])? {
            "=" => statement.value = Some(Box::new(Expression::parse(lexer)?)),
            ";" => return Ok(statement),
            _ => unreachable!(),
        }

        lexer.expect_punctuation([";"])?;
        Ok(statement)
    }
}