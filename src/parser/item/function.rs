use crate::{lexer::{TokenStream, Token, punctuation::Punctuation}, parser::UnexpectedTokenError};

use crate::parser::{expressions::Identifier, ParserError, Statement, Delimiter};

/// A function is a set of statements to perform a specific task.
/// 
/// `fn NAME(NAME: TYPE, ...) -> RETURN_TYPE`
#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub return_type: Option<Identifier>,
    pub body: Vec<Statement>,
}

/// A parameter represents a value that the function expects you to pass when you call it.
/// 
/// `NAME: TYPE`
#[derive(Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: Identifier,
    pub type_: Identifier,
}

impl Function {
    /// Parse function from token stream. `fn` keyword is expected to be consumed beforehand.
    pub fn parse(lexer: &mut TokenStream) -> Result<Function, ParserError> {
        let name = Identifier::parse(lexer)?;
        lexer.expect_punctuation(["("])?;
        let params = Self::parse_params(lexer)?;
        let return_type = Self::parse_return_type(lexer)?;
        let body = Statement::parse_block(lexer)?;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    /// Parse parameters. Opening parenthesis (`(`) is expected to be consumed beforehand.
    fn parse_params(lexer: &mut TokenStream) -> Result<Vec<Parameter>, ParserError> {
        let mut params = Vec::new();
        loop {
            let name = match lexer.next_some()? {
                Token::Identifier(ident) => Identifier(ident),
                Token::Punctuation(Punctuation(")")) => break,
                token => return Err(UnexpectedTokenError::UnexpectedToken(token).into())
            };
            lexer.expect_punctuation([":"])?;
            let type_ = Identifier::parse(lexer)?;
            params.push(Parameter { name, type_ });

            if ")" == lexer.expect_punctuation([")", ","])? {
                break
            }
        }
        Ok(params)
    }

    /// Try to parse return type if any. Consumes opening brace `{` which is required for function body.
    fn parse_return_type(lexer: &mut TokenStream) -> Result<Option<Identifier>, ParserError> {
        match lexer.next_some()? {
            Token::Punctuation(Punctuation("->")) => {
                let return_type = Identifier::parse(lexer)?;
                lexer.expect_punctuation(["{"])?;
                Ok(Some(return_type))
            },
            Token::Punctuation(Punctuation("{")) => Ok(None),
            token => Err(UnexpectedTokenError::UnexpectedToken(token).into()),
        }
    }
}
