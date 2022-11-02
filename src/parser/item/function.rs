use crate::{lexer::{TokenStream, Token, punctuation::Punctuation}, parser::UnexpectedTokenError};

use crate::parser::{expressions::Identifier, ParserError, Statement, Delimiter};

/// fn NAME(NAME: TYPE, ...) -> RETURN_TYPE
#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub params: Vec<FunctionParameter>,
    pub return_type: Option<Identifier>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: Identifier,
    pub type_: Identifier,
}

impl Function {
    pub fn parse(lexer: &mut TokenStream) -> Result<Function, ParserError> {
        let name = Identifier::parse(lexer)?;
        lexer.expect_punctuation(&["("])?;
        let params = Self::parse_params(lexer)?;
        let return_type = Self::parse_return_type(lexer)?;
        let body = Statement::parse_block(lexer, Delimiter::Brace)?;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(lexer: &mut TokenStream) -> Result<Vec<FunctionParameter>, ParserError> {
        let mut params = Vec::new();
        loop {
            let token = lexer.next_some()?;
            if let Token::Punctuation(Punctuation(")")) = token {
                break;
            }
            let name = Identifier::parse(lexer)?;
            lexer.expect_punctuation(&[":"])?;
            let type_ = Identifier::parse(lexer)?;
            params.push(FunctionParameter { name, type_ });

            match lexer.next_some()? {
                Token::Punctuation(Punctuation(")")) => break,
                Token::Punctuation(Punctuation(",")) => { },
                token => return Err(UnexpectedTokenError::UnexpectedToken(token).into())
            }
        }
        Ok(params)
    }

    fn parse_return_type(lexer: &mut TokenStream) -> Result<Option<Identifier>, ParserError> {
        match lexer.next_some()? {
            Token::Punctuation(Punctuation("->")) => {
                let return_type = Identifier::parse(lexer)?;
                lexer.expect_punctuation(&["{"])?;
                Ok(Some(return_type))
            },
            Token::Punctuation(Punctuation("{")) => return Ok(None),
            token => return Err(UnexpectedTokenError::UnexpectedToken(token).into()),
        }
    }
}
