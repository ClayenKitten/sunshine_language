mod operator;

pub use operator::{UnaryOp, BinaryOp};

use crate::lexer::{number::Number, Token, TokenStream};

use super::{ParserError, UnexpectedTokenError, Statement};

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Vec<Statement>),
    /// Group is a single parenthized expression.
    Group(Box<Expression>),

    Unary(UnaryOp),
    Binary(BinaryOp),
    
    If(If),
    While(While),
    For(For),
    Identifier(Identifier),
    Literal(Literal),
    Assignment(Assignment),
    FunctionCall(FunctionCall),
}

impl Expression {
    pub fn parse(lexer: &mut TokenStream) -> Result<Expression, ParserError> {
        todo!()
    }
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
