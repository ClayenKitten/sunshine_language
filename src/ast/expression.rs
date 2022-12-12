use crate::lexer::{number::Number, punctuation::Punctuation};

use super::{Identifier, statement::Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Block),

    If(If),
    While(While),
    For(For),

    Identifier(Identifier),
    Literal(Literal),

    Unary {
        op: Punctuation,
        value: Box<Expression>,
    },
    Binary {
        op: Punctuation,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    FunctionCall(FunctionCall),
    Variable(Identifier),
}

impl Expression {
    /// Check if that expression is block expression.
    ///
    /// Block expressions end with a right brace and don't require to be followed by a semicolon to
    /// be accounted as expression statement.
    pub fn is_block_expression(&self) -> bool {
        matches!(
            self,
            Expression::Block(_) | Expression::If(_) | Expression::While(_) | Expression::For(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Number(Number),
    String(String),
    Boolean(bool),
}

/// Block is an expression that consists of a number of statements and an optional final expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}

/// NAME(PARAMS, ...)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub params: Vec<Expression>,
}

/// if CONDITION { BODY } else { ELSE_BODY }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub condition: Box<Expression>,
    pub body: Block,
    pub else_body: Option<Block>,
}

/// while CONDITION { BODY }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct While {
    pub condition: Box<Expression>,
    pub body: Block,
}

/// for VAR in EXPR { BODY }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct For {
    pub var: Identifier,
    pub expr: Box<Expression>,
    pub body: Block,
}
