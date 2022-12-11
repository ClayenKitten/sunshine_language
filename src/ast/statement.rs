use super::expression::{Expression, Identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    ExpressionStatement(Expression),
    LetStatement(LetStatement),
    Break,
}

/// let VAR: TYPE = VALUE;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Identifier,
    pub type_: Option<Identifier>,
    pub value: Option<Box<Expression>>,
}
