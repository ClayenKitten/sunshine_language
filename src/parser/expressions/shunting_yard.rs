use std::collections::VecDeque;

use crate::{lexer::{punctuation::Punctuation, Token, TokenStream}, parser::{ParserError, UnexpectedTokenError}};

use super::Expression;

/// An expression that stores a sequence of operands and operators.
#[derive(Debug, PartialEq, Eq)]
pub struct ReversePolishExpr(VecDeque<PolishEntry>);

impl ReversePolishExpr {
    /// Parse binary expression.
    /// 
    /// Parsing continues until "stopper" punctuation met or error occur.
    pub fn parse(lexer: &mut TokenStream) -> Result<Self, ParserError> {
        let mut output = VecDeque::<PolishEntry>::new();
        let mut op_stack = Vec::<Operator>::new();

        let mut is_last_token_an_operand = false;

        loop {
            match lexer.peek()? {
                Token::Punctuation(punc) if punc.0 == "(" => {
                    lexer.next()?;

                    is_last_token_an_operand = false;
                    op_stack.push(Operator::LeftParenthesis)
                }
                Token::Punctuation(punc) if punc.0 == ")" => {
                    lexer.next()?;

                    while let Some(top_op) = op_stack.last() {
                        if top_op == &Operator::LeftParenthesis {
                            break;
                        }
                        output.push_back(PolishEntry::Operator(op_stack.pop().unwrap()));
                    }
                    
                    // Either `op_stack` is empty or left parenthesis is on the top at that point.
                    if op_stack.pop().is_none() {
                        // Expected left parenthesis
                        return Err(UnexpectedTokenError::TokenMismatch.into())
                    }
                    is_last_token_an_operand = true;
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
                    let priority = punc.binary_priority().unwrap_or(u8::MAX);
    
                    while let Some(top_op) = op_stack.last() {
                        if top_op == &Operator::LeftParenthesis {
                            break;
                        }
                        if let Operator::Binary { priority: top_priority, .. } = top_op {
                            if *top_priority < priority {
                                break;
                            }
                        }
                        output.push_back(PolishEntry::Operator(op_stack.pop().unwrap()));
                    }
                    if arity == 2 {
                        op_stack.push(Operator::Binary { punc, priority: punc.binary_priority().unwrap() })
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
            output.push_back(PolishEntry::Operator(op));
        }
        
        Ok(ReversePolishExpr(output))
    }

    
}

#[derive(Debug, PartialEq, Eq)]
enum PolishEntry {
    Operand(Expression),
    Operator(Operator),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operator {
    Unary { punc: Punctuation },
    Binary { punc: Punctuation, priority: u8 },
    LeftParenthesis,
}
