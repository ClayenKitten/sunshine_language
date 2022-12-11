pub mod r#let;

use self::r#let::LetStatement;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    ExpressionStatement(Expression),
    LetStatement(LetStatement),
    Break,
}

/// Block is an expression that consists of a number of statements and an optional final expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}
