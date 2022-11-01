use strum::EnumDiscriminants;
use crate::lexer::{punctuation::{Operator, Punctuation}, number::Number, Token, TokenKind, TokenStream};

use super::{ParserError, UnexpectedTokenError, Statement, Delimiter};

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    WithBlock(ExpressionWithBlock),
    WithoutBlock(ExpressionWithoutBlock),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionWithBlock {
    Block(Vec<Statement>),
    If(If),
    While(While),
    For(For),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionWithoutBlock {
    Identifier(Identifier),
    Literal(Literal),
    Assignment(Assignment),
    FunctionCall(FunctionCall),
}

/// Identifier is name of type, variable or function.
#[derive(Debug, PartialEq, Eq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn parse(lexer: &mut TokenStream) -> Result<Identifier, ParserError> {
        let token = lexer.next_some()?;
        if let Token::Identifier(ident) = token {
            Ok(Identifier(ident))
        } else {
            Err(UnexpectedTokenError::UnexpectedToken(token).into())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literal {
    Number(Number),
    String(String),
    Boolean(bool),
}

/// let VAR: TYPE = VALUE
#[derive(Debug, PartialEq, Eq)]
pub struct LetStatement {
    pub var: Identifier,
    pub type_: Option<Identifier>,
    pub value: Option<Box<Expression>>,
}

/// VAR = VALUE
#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub var: Identifier,
    pub value: Box<Expression>,
}

/// NAME(PARAMS, ...)
#[derive(Debug, PartialEq, Eq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub params: Vec<Expression>,
}

/// if CONDITION { BODY } else { ELSE_BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct If {
    pub condition: Box<Expression>,
    pub body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

/// while CONDITION { BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct While {
    pub condition: Box<Expression>,
    pub body: Vec<Statement>,
}

/// for VAR in EXPR { BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct For {
    pub var: Identifier,
    pub expr: Box<Expression>,
    pub body: Vec<Statement>,
}
