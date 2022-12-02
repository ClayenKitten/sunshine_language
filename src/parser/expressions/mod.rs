use std::collections::VecDeque;
use crate::lexer::{number::Number, Token, TokenStream, punctuation::Punctuation, keyword::Keyword};

use super::{ParserError, UnexpectedTokenError, Statement};

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Vec<Statement>),
    
    /// Expression with operators stored in reverse polish notation.
    Polish(VecDeque<PolishEntry>),
    
    If(If),
    While(While),
    For(For),

    Identifier(Identifier),
    Literal(Literal),
    Assignment(Assignment),
    
    FunctionCall(FunctionCall),
    Variable(Identifier),
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolishEntry {
    Operand(Expression),
    Operator(Operator),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Operator { punc: Punctuation, arity: u8 }

impl Expression {
    pub fn parse(lexer: &mut TokenStream) -> Result<Expression, ParserError> {
        Self::parse_binary(lexer)
            .map(|expr| expr.0)
    }

    /// Parse binary expression.
    /// 
    /// Parsing continues until "stopper" punctuation met or error occur.
    /// 
    /// # Returns
    /// 
    /// Both parsed expression and stopper are returned.
    fn parse_binary(lexer: &mut TokenStream) -> Result<(Expression, Punctuation), ParserError> {
        let mut output = VecDeque::<PolishEntry>::new();
        let mut op_stack = Vec::<Operator>::new();

        let mut is_last_token_an_operand = false;

        let stopper = loop {
            match lexer.peek()? {
                Token::Punctuation(punc) if punc.0 == "(" => {
                    lexer.next()?;
                    op_stack.push(Operator { punc, arity: 0 })
                }
                Token::Punctuation(punc) if punc.0 == ")" => {
                    lexer.next()?;
                    while let Some(top_op) = op_stack.last() {
                        if top_op.punc == Punctuation("(") {
                            break;
                        }
                        output.push_back(PolishEntry::Operator(op_stack.pop().unwrap()));
                    }

                    // Either `op_stack` is empty or left parenthesis is on the top at that point.
                    if op_stack.pop().is_none() {
                        // Expected left parenthesis
                        return Err(UnexpectedTokenError::TokenMismatch.into())
                    }
                }
                Token::Punctuation(punc) if punc.is_stopper() => {
                    lexer.next()?;
                    break punc;
                }
                Token::Punctuation(punc) if punc.is_operator() => {
                    lexer.next()?;

                    let arity = Self::operator_arity(punc, is_last_token_an_operand)?;
                    is_last_token_an_operand = false;
                    let priority = punc.binary_priority().unwrap_or(u8::MAX);
    
                    while let Some(top_op) = op_stack.last() {
                        if top_op.punc == Punctuation("(") {
                            break;
                        }
                        if top_op.punc.binary_priority().unwrap() < priority {
                            break;
                        }
                        output.push_back(PolishEntry::Operator(op_stack.pop().unwrap()));
                    }
                    op_stack.push(Operator { punc, arity })
                }
                _ => {
                    let operand = Self::parse_operand(lexer)?;
                    output.push_back(PolishEntry::Operand(operand));
                    is_last_token_an_operand = true;
                }
            }
        };

        while let Some(op) = op_stack.pop() {
            output.push_back(PolishEntry::Operator(op));
        }
        
        Ok((Expression::Polish(output), stopper))
    }

    fn operator_arity(op: Punctuation, is_last_token_an_operand: bool) -> Result<u8, ParserError> {
        if is_last_token_an_operand && op.is_binary_operator() {
            Ok(2)
        } else if !is_last_token_an_operand && op.is_unary_operator() {
            Ok(1)
        } else {
            Err(UnexpectedTokenError::TokenMismatch.into())
        }
    }

    /// Parse a single operand
    fn parse_operand(lexer: &mut TokenStream) -> Result<Expression, ParserError> {
        let token = match lexer.next()? {
            Token::Punctuation(Punctuation("{")) => {
                Expression::Block(Statement::parse_block(lexer)?)
            }

            Token::Punctuation(_) => {
                return Err(UnexpectedTokenError::TokenMismatch.into())
            }

            Token::Number(num) => Expression::Literal(Literal::Number(num)),
            Token::String(str) => Expression::Literal(Literal::String(str)),

            Token::Keyword(kw) => {
                match kw {
                    Keyword::If => todo!(),
                    Keyword::While => todo!(),
                    Keyword::For => todo!(),
                    Keyword::True => Expression::Literal(Literal::Boolean(true)),
                    Keyword::False => Expression::Literal(Literal::Boolean(false)),
                    _ => return Err(UnexpectedTokenError::TokenMismatch.into()),
                }
            }

            Token::Identifier(ident) => {
                Self::maybe_function_call(lexer, Identifier(ident))?
            },

            Token::Eof => return Err(ParserError::UnexpectedEof),
        };
        Ok(token)
    }

    /// Try to wrap provided identifier in function call.
    fn maybe_function_call(lexer: &mut TokenStream, name: Identifier) -> Result<Expression, ParserError> {
        if lexer.expect_punctuation(["("]).is_ok() {
            let mut params = Vec::new();
            loop {
                let (expr, stopper) = Self::parse_binary(lexer)?;
                params.push(expr);

                match stopper.0 {
                    ")" => return Ok(Expression::FunctionCall(FunctionCall { name, params })),
                    "," => { },
                    _ => return Err(UnexpectedTokenError::TokenMismatch.into()),
                }
            }
        } else {
            Ok(Expression::Variable(name))
        }
    }
}

/// Identifier is name of type, variable or function.
#[derive(Debug, PartialEq, Eq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn parse(lexer: &mut TokenStream) -> Result<Identifier, ParserError> {
        let token = lexer.next_some()?;
        if let Token::Identifier(ident) = token {
            Ok(Identifier(ident))
        } else {
            Err(UnexpectedTokenError::UnexpectedToken(token).into())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literal {
    Number(Number),
    String(String),
    Boolean(bool),
}

/// VAR = VALUE
#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub var: Identifier,
    pub value: Box<Expression>,
}

/// NAME(PARAMS, ...)
#[derive(Debug, PartialEq, Eq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub params: Vec<Expression>,
}

/// if CONDITION { BODY } else { ELSE_BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct If {
    pub condition: Box<Expression>,
    pub body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

/// while CONDITION { BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct While {
    pub condition: Box<Expression>,
    pub body: Vec<Statement>,
}

/// for VAR in EXPR { BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct For {
    pub var: Identifier,
    pub expr: Box<Expression>,
    pub body: Vec<Statement>,
}
