use crate::lexer::punctuation::Punctuation;

use super::Expression;

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryOp {
    pub operator: Punctuation,
    pub operand: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryOp {
    pub operator: Punctuation,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}
