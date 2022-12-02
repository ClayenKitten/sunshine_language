mod shunting_yard;

use std::collections::VecDeque;
use crate::lexer::{number::Number, Token, TokenStream, punctuation::Punctuation, keyword::Keyword};

use self::shunting_yard::ReversePolishExpr;

use super::{ParserError, UnexpectedTokenError, Statement};

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Vec<Statement>),
    
    /// Expression with operators stored in reverse polish notation.
    Polish(ReversePolishExpr),
    
    If(If),
    While(While),
    For(For),

    Identifier(Identifier),
    Literal(Literal),
    Assignment(Assignment),
    
    FunctionCall(FunctionCall),
    Variable(Identifier),
}

impl Expression {
    pub fn parse(lexer: &mut TokenStream) -> Result<Expression, ParserError> {
        shunting_yard::ReversePolishExpr::parse(lexer)
            .map(|expr| Self::Polish(expr.0))
    }

    /// Parse a single operand
    fn parse_operand(lexer: &mut TokenStream) -> Result<Expression, ParserError> {
        let token = match lexer.next()? {
            Token::Punctuation(Punctuation("{")) => {
                Expression::Block(Statement::parse_block(lexer)?)
            }

            Token::Punctuation(_) => {
                return Err(UnexpectedTokenError::TokenMismatch.into())
            }

            Token::Number(num) => Expression::Literal(Literal::Number(num)),
            Token::String(str) => Expression::Literal(Literal::String(str)),

            Token::Keyword(kw) => {
                match kw {
                    Keyword::If => todo!(),
                    Keyword::While => todo!(),
                    Keyword::For => todo!(),
                    Keyword::True => Expression::Literal(Literal::Boolean(true)),
                    Keyword::False => Expression::Literal(Literal::Boolean(false)),
                    _ => return Err(UnexpectedTokenError::TokenMismatch.into()),
                }
            }

            Token::Identifier(ident) => {
                Self::maybe_function_call(lexer, Identifier(ident))?
            },

            Token::Eof => return Err(ParserError::UnexpectedEof),
        };
        Ok(token)
    }

    /// Try to wrap provided identifier in function call.
    fn maybe_function_call(lexer: &mut TokenStream, name: Identifier) -> Result<Expression, ParserError> {
        if lexer.expect_punctuation(["("]).is_ok() {
            let mut params = Vec::new();
            loop {
                let (expr, stopper) = shunting_yard::ReversePolishExpr::parse(lexer)
                    .map(|expr| (Self::Polish(expr.0), expr.1))?;
                params.push(expr);

                match stopper.0 {
                    ")" => return Ok(Expression::FunctionCall(FunctionCall { name, params })),
                    "," => { },
                    _ => return Err(UnexpectedTokenError::TokenMismatch.into()),
                }
            }
        } else {
            Ok(Expression::Variable(name))
        }
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
