mod function;
mod r#struct;

use crate::{lexer::{Lexer, Token, keyword::Keyword}, parser::UnexpectedTokenError};

use super::ParserError;

pub use self::{r#struct::{Struct, Field}, function::{Function, Parameter}};

/// An Item is a static component of the package.
#[derive(Debug, PartialEq, Eq)]
pub enum Item {
    Struct(Struct),
    Function(Function),
}

impl Item {
    pub fn parse(lexer: &mut Lexer) -> Result<Item, ParserError> {
        Ok(match lexer.next()? {
            Token::Keyword(Keyword::Fn) => Item::Function(Function::parse(lexer)?),
            Token::Keyword(Keyword::Struct) => Item::Struct(Struct::parse(lexer)?),
            token => return Err(UnexpectedTokenError::UnexpectedToken(token).into()),
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Item::Struct(s) => &s.name.0,
            Item::Function(f) => &f.name.0,
        }
    }
}