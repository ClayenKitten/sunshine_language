use crate::lexer::punctuation::Punctuation;

use super::{expression::Expression, Identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    ExprStmt(Expression),
    LetStmt(LetStatement),
    Assignment(Assignment),
    Return(Expression),
    Break,
}

/// let VAR: TYPE = VALUE;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Identifier,
    pub type_: Option<Identifier>,
    pub value: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub assignee: Identifier,
    pub operator: Punctuation,
    pub value: Expression,
}
