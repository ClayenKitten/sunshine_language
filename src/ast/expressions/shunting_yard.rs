use std::collections::VecDeque;

use crate::{lexer::{punctuation::Punctuation, Token, Lexer}, ast::ParserError};

use super::Expression;

/// An expression that stores a sequence of operands and operators.
#[derive(Debug, PartialEq, Eq)]
pub struct ReversePolishNotation(VecDeque<PolishEntry>);

impl ReversePolishNotation {
    /// Parse binary expression.
    /// 
    /// Parsing continues until "stopper" punctuation met or error occur.
    pub fn parse(lexer: &mut Lexer) -> Result<Self, ParserError> {
        let mut output = VecDeque::<PolishEntry>::new();
        let mut op_stack = Vec::<Operator>::new();

        let mut is_last_token_an_operand = false;

        loop {
            match lexer.peek()? {
                Token::Punctuation(punc) if punc.0 == "(" => {
                    lexer.next()?;
                    is_last_token_an_operand = false;
                    op_stack.push(Operator::LeftParenthesis);
                }
                Token::Punctuation(punc) if punc.0 == ")" => {
                    while let Some(top_op) = op_stack.last() {
                        if top_op == &Operator::LeftParenthesis {
                            break;
                        }
                        output.push_back(op_stack.pop().unwrap().try_into().unwrap());
                    }
                    
                    // Either `op_stack` is empty or left parenthesis is on the top at that point.
                    if op_stack.pop().is_none() {
                        break;
                    }
                    is_last_token_an_operand = true;
                    lexer.next()?;
                }
                Token::Punctuation(punc) if punc.is_operator() => {
                    let arity = if is_last_token_an_operand && punc.is_binary_operator() {
                        2
                    } else if !is_last_token_an_operand && punc.is_unary_operator() {
                        1
                    } else {
                        break;
                    };
                    lexer.next()?;
                    
                    is_last_token_an_operand = false;
                    let priority = punc.priority();
    
                    while let Some(top_op) = op_stack.last() {
                        if top_op == &Operator::LeftParenthesis {
                            break;
                        }
                        if let Operator::Binary { priority: top_priority, .. } = top_op {
                            if *top_priority < priority {
                                break;
                            }
                        }
                        output.push_back(op_stack.pop().unwrap().try_into().unwrap());
                    }
                    if arity == 2 {
                        op_stack.push(Operator::Binary { punc, priority: punc.priority() })
                    } else if arity == 1 {
                        op_stack.push(Operator::Unary { punc })
                    }
                }
                _ => {
                    if is_last_token_an_operand {
                        break;
                    }
                    let operand = Expression::parse_operand(lexer)?;
                    output.push_back(PolishEntry::Operand(operand));
                    is_last_token_an_operand = true;
                }
            }
        };

        while let Some(op) = op_stack.pop() {
            output.push_back(op.try_into().unwrap());
        }
        
        Ok(ReversePolishNotation(output))
    }

    /// Convert an RPN to expression tree.
    pub fn into_tree(mut self) -> Expression {
        Self::get_node(&mut self.0)
    }

    fn get_node(buf: &mut VecDeque<PolishEntry>) -> Expression {
        match buf.pop_back().unwrap() {
            PolishEntry::Operand(expr) => {
                expr
            }
            PolishEntry::UnaryOperator(punc) => {
                let value = Box::new(Self::get_node(buf));
                Expression::Unary { op: punc, value }
            }
            PolishEntry::BinaryOperator(punc) => {
                let right = Box::new(Self::get_node(buf));
                let left = Box::new(Self::get_node(buf));
                Expression::Binary { op: punc, left, right }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PolishEntry {
    Operand(Expression),
    UnaryOperator(Punctuation),
    BinaryOperator(Punctuation),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operator {
    Unary { punc: Punctuation },
    Binary { punc: Punctuation, priority: u8 },
    LeftParenthesis,
}

impl TryFrom<Operator> for PolishEntry {
    type Error = ();

    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        match value {
            Operator::Unary { punc } => Ok(PolishEntry::UnaryOperator(punc)),
            Operator::Binary { punc, .. } => Ok(PolishEntry::BinaryOperator(punc)),
            Operator::LeftParenthesis => Err(()),
        }
    }
}
