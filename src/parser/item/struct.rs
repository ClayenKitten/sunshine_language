use crate::{parser::{expressions::Identifier, ParserError}, lexer::{TokenStream, Token, punctuation::Punctuation}};

#[derive(Debug, PartialEq, Eq)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub name: Identifier,
    pub type_: Identifier,
}

impl Struct {
    pub fn parse(lexer: &mut TokenStream) -> Result<Struct, ParserError> {
        let name = Identifier::parse(lexer)?;
        lexer.expect(|token| matches!(token, Token::Punctuation(Punctuation("{"))));
        todo!();
    }
}
