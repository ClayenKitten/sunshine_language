use super::{Identifier, expression::Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    ExprStmt(Expression),
    LetStmt(LetStatement),
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
