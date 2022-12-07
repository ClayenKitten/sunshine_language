mod module;
mod function;
mod r#struct;

use crate::{lexer::{Lexer, Token, keyword::Keyword}, parser::UnexpectedTokenError};

use super::ParserError;

pub use self::{module::Module, r#struct::{Struct, Field}, function::{Function, Parameter}};

/// An Item is a static component of the package.
#[derive(Debug, PartialEq, Eq)]
pub enum Item {
    Module(Module),
    Struct(Struct),
    Function(Function),
}

impl Item {
    pub fn parse(lexer: &mut Lexer) -> Result<Item, ParserError> {
        let start = lexer.location;
        Ok(match lexer.next()? {
            Token::Keyword(Keyword::Fn) => Item::Function(Function::parse(lexer)?),
            Token::Keyword(Keyword::Struct) => Item::Struct(Struct::parse(lexer)?),
            Token::Keyword(Keyword::Mod) => Item::Module(Module::parse(lexer)?),
            token => {
                lexer.error_reporter.error()
                    .message(String::from("expected an item"))
                    .starts_at(start)
                    .ends_at(lexer.location)
                    .report();
                return Err(UnexpectedTokenError::UnexpectedToken(token).into())
            },
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Item::Module(m) => &m.name.0,
            Item::Struct(s) => &s.name.0,
            Item::Function(f) => &f.name.0,
        }
    }
}