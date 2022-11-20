use crate::lexer::punctuation::Punctuation;

use super::Expression;

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryOp {
    operator: Punctuation,
    operand: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryOp {
    operator: Punctuation,
    left: Box<Expression>,
    right: Box<Expression>,
}
