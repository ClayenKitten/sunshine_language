use std::collections::VecDeque;

use crate::{
    ast::expression::Expression,
    lexer::punctuation::{BinaryOp, UnaryOp},
};

use super::infix_notation::{InfixEntry, InfixExpr};

/// A sequence of operands and operators in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
#[derive(Debug, PartialEq, Eq)]
pub struct ReversePolishExpr(VecDeque<PolishEntry>);

impl ReversePolishExpr {
    pub fn from_infix(infix: InfixExpr) -> Self {
        let mut output = VecDeque::<PolishEntry>::with_capacity(infix.0.capacity());
        let mut op_stack = Vec::<Operator>::new();

        for entry in infix.0 {
            match entry {
                InfixEntry::Operand(operand) => {
                    output.push_back(PolishEntry::Operand(operand));
                }
                InfixEntry::UnaryOperator(op) => op_stack.push(Operator::Unary(op)),
                InfixEntry::BinaryOperator(op) => {
                    while let Some(top_op) = op_stack.last() {
                        let top_priority = match top_op {
                            Operator::Unary(_) => 0,
                            Operator::Binary(op) => op.priority(),
                            Operator::LeftParenthesis => break,
                        };
                        if top_priority < op.priority() {
                            break;
                        }
                        output.push_back(op_stack.pop().unwrap().try_into().unwrap());
                    }
                    op_stack.push(Operator::Binary(op));
                }
                InfixEntry::LeftParenthesis => op_stack.push(Operator::LeftParenthesis),
                InfixEntry::RightParenthesis => {
                    while let Some(top_op) = op_stack.last() {
                        if top_op == &Operator::LeftParenthesis {
                            break;
                        }
                        output.push_back(op_stack.pop().unwrap().try_into().unwrap());
                    }

                    // Either `op_stack` is empty or left parenthesis is on the top at that point.
                    if op_stack.pop().is_none() {
                        panic!(":(");
                    }
                }
            }
        }

        while let Some(op) = op_stack.pop() {
            output.push_back(op.try_into().unwrap());
        }

        ReversePolishExpr(output)
    }
    /// Convert an RPN to expression tree.
    pub fn into_tree(mut self) -> Expression {
        Self::get_node(&mut self.0)
    }

    fn get_node(buf: &mut VecDeque<PolishEntry>) -> Expression {
        match buf.pop_back().unwrap() {
            PolishEntry::Operand(expr) => expr,
            PolishEntry::UnaryOperator(punc) => {
                let value = Box::new(Self::get_node(buf));
                Expression::Unary { op: punc, value }
            }
            PolishEntry::BinaryOperator(punc) => {
                let right = Box::new(Self::get_node(buf));
                let left = Box::new(Self::get_node(buf));
                Expression::Binary {
                    op: punc,
                    left,
                    right,
                }
            }
        }
    }
}

impl From<InfixExpr> for ReversePolishExpr {
    fn from(val: InfixExpr) -> Self {
        ReversePolishExpr::from_infix(val)
    }
}

/// An entry of RPN expression: operand or operator (unary or binary).
#[derive(Debug, PartialEq, Eq)]
pub enum PolishEntry {
    Operand(Expression),
    UnaryOperator(UnaryOp),
    BinaryOperator(BinaryOp),
}

impl TryFrom<Operator> for PolishEntry {
    type Error = ();

    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        match value {
            Operator::Unary(op) => Ok(PolishEntry::UnaryOperator(op)),
            Operator::Binary(op) => Ok(PolishEntry::BinaryOperator(op)),
            Operator::LeftParenthesis => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Operator {
    Unary(UnaryOp),
    Binary(BinaryOp),
    LeftParenthesis,
}
