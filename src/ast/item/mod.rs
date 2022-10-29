use crate::lexer::TokenStream;

use super::{expressions::Identifier, ParserError, Statement};

#[derive(Debug, PartialEq, Eq)]
pub enum Item {
    Struct(Struct),
    Function(Function),
}

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

/// fn NAME(NAME: TYPE, ...) -> RETURN_TYPE
#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub params: Vec<FunctionParameter>,
    pub body: Vec<Statement>,
    pub return_type: Option<Identifier>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: Identifier,
    pub type_: Identifier,
}

impl Item {
    pub fn parse(lexer: &mut TokenStream) -> Result<Item, ParserError> {
        todo!();
    }
}
