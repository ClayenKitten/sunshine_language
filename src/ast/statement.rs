use crate::lexer::operator::AssignOp;

use super::{expression::Expression, Identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    ExprStmt(Expression),
    LetStmt(LetStatement),
    Assignment {
        assignee: Identifier,
        operator: AssignOp,
        expression: Expression,
    },
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
