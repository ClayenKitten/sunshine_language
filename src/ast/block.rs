//! Block is an expression with attached SymbolTable for its contents.

use super::{expression::Expression, statement::Statement};

/// Block is an expression that consists of a number of statements and an optional final expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}
