mod function;
mod r#struct;

use crate::{lexer::{TokenStream, Token, keyword::Keyword}, parser::UnexpectedTokenError};

use super::ParserError;

pub use self::{r#struct::{Struct, Field}, function::{Function, Parameter}};

#[derive(Debug, PartialEq, Eq)]
pub enum Item {
    Struct(Struct),
    Function(Function),
}

impl Item {
    pub fn parse(lexer: &mut TokenStream) -> Result<Item, ParserError> {
        Ok(match lexer.next()? {
            Token::Keyword(Keyword::Fn) => Item::Function(Function::parse(lexer)?),
            Token::Keyword(Keyword::Struct) => Item::Struct(Struct::parse(lexer)?),
            token => return Err(UnexpectedTokenError::UnexpectedToken(token).into()),
        })
    }
}