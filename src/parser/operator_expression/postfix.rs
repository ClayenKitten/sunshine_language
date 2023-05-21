//! Postfix operator expressions.
use std::collections::VecDeque;

use crate::{
    ast::expression::Expression,
    error::CompilerError,
    lexer::operator::{BinaryOp, UnaryOp},
};

use super::{
    infix::{InfixEntry, InfixNotation},
    MaybeAssignment, Tree,
};

/// A sequence of operands and operators in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
pub type PostfixNotation = MaybeAssignment<VecDeque<PostfixEntry>>;

impl PostfixNotation {
    /// Converts from infix to postfix notation.
    pub fn from_infix(infix: InfixNotation) -> Self {
        infix.map_expr(|entries| {
            let mut output = VecDeque::<PostfixEntry>::with_capacity(entries.capacity());
            let mut op_stack = Vec::<Operator>::with_capacity(4);

            for entry in entries {
                match entry {
                    InfixEntry::Operand(operand) => {
                        output.push_back(PostfixEntry::Operand(operand));
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
                            panic!("Operator stack should be empty");
                        }
                    }
                }
            }

            while let Some(op) = op_stack.pop() {
                output.push_back(op.try_into().unwrap());
            }

            output
        })
    }

    /// Converts from postfix notation to tree.
    pub fn into_tree(self) -> Tree {
        match self {
            PostfixNotation::Expression(mut expression) => {
                MaybeAssignment::Expression(Self::get_node(&mut expression))
            }
            PostfixNotation::Assignment {
                assignee,
                operator,
                mut expression,
            } => MaybeAssignment::Assignment {
                assignee,
                operator,
                expression: Self::get_node(&mut expression),
            },
        }
    }

    /// Converts from postfix notation to expression tree, issuing a error if it is not possible.
    pub fn into_expression(self) -> Result<Expression, CompilerError> {
        if let PostfixNotation::Expression(mut expression) = self {
            Ok(Self::get_node(&mut expression))
        } else {
            Err(CompilerError)
        }
    }

    fn get_node(buf: &mut VecDeque<PostfixEntry>) -> Expression {
        match buf.pop_back().unwrap() {
            PostfixEntry::Operand(expr) => expr,
            PostfixEntry::UnaryOperator(punc) => {
                let value = Box::new(Self::get_node(buf));
                Expression::Unary { op: punc, value }
            }
            PostfixEntry::BinaryOperator(punc) => {
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

/// An entry of postfix expression: operand or operator (unary or binary).
#[derive(Debug, PartialEq, Eq)]
pub enum PostfixEntry {
    Operand(Expression),
    UnaryOperator(UnaryOp),
    BinaryOperator(BinaryOp),
}

impl TryFrom<Operator> for PostfixEntry {
    type Error = ();

    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        match value {
            Operator::Unary(op) => Ok(PostfixEntry::UnaryOperator(op)),
            Operator::Binary(op) => Ok(PostfixEntry::BinaryOperator(op)),
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
