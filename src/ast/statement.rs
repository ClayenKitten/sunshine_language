use super::expressions::{Expression, Identifier};

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

/// let VAR: TYPE = VALUE;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Identifier,
    pub type_: Option<Identifier>,
    pub value: Option<Box<Expression>>,
}
