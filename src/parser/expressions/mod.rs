mod shunting_yard;

use crate::lexer::{number::Number, Token, Lexer, punctuation::Punctuation, keyword::Keyword};

use super::{ParserError, UnexpectedTokenError, statement::Block};

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Block),

    If(If),
    While(While),
    For(For),

    Identifier(Identifier),
    Literal(Literal),

    Unary {
        op: Punctuation,
        value: Box<Expression>
    },
    Binary {
        op: Punctuation,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    
    FunctionCall(FunctionCall),
    Variable(Identifier),
}

impl Expression {
    pub fn parse(lexer: &mut Lexer) -> Result<Expression, ParserError> {
        shunting_yard::ReversePolishNotation::parse(lexer)
            .map(|expr| expr.into_tree())
    }

    /// Parse a single operand
    fn parse_operand(lexer: &mut Lexer) -> Result<Expression, ParserError> {
        let token = match lexer.next()? {
            Token::Punctuation(Punctuation("{")) => {
                Expression::Block(Block::parse(lexer)?)
            }

            Token::Punctuation(_) => {
                return Err(UnexpectedTokenError::TokenMismatch.into())
            }

            Token::Number(num) => Expression::Literal(Literal::Number(num)),
            Token::String(str) => Expression::Literal(Literal::String(str)),

            Token::Keyword(kw) => {
                match kw {
                    Keyword::If => Expression::If(If::parse(lexer)?),
                    Keyword::While => Expression::While(While::parse(lexer)?),
                    Keyword::For => Expression::For(For::parse(lexer)?),
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
    fn maybe_function_call(lexer: &mut Lexer, name: Identifier) -> Result<Expression, ParserError> {
        if lexer.consume_punctuation("(")? {
            let mut params = Vec::new();
            loop {
                params.push(Expression::parse(lexer)?);
                if lexer.consume_punctuation(")")? {
                    return Ok(Expression::FunctionCall(FunctionCall { name, params }));
                } else if lexer.consume_punctuation(",")? {
                    
                } else {
                    return Err(UnexpectedTokenError::TokenMismatch.into());
                }
            }
        } else {
            Ok(Expression::Variable(name))
        }
    }

    /// Check if that expression is block expression.
    /// 
    /// Block expressions end with a right brace and don't require to be followed by a semicolon to
    /// be accounted as expression statement.
    pub fn is_block_expression(&self) -> bool {
        matches!(
            self,
            Expression::Block(_) |
            Expression::If(_) |
            Expression::While(_) |
            Expression::For(_)
        )
    }
}

/// Identifier is name of type, variable or function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Number(Number),
    String(String),
    Boolean(bool),
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
    pub body: Block,
    pub else_body: Option<Block>,
}

impl If {
    pub fn parse(lexer: &mut Lexer) -> Result<If, ParserError> {
        let condition = Box::new(Expression::parse(lexer)?);
        lexer.expect_punctuation("{")?;
        let body = Block::parse(lexer)?;
        let else_body = if let Token::Keyword(Keyword::Else) = lexer.peek()? {
            let _ = lexer.next();
            lexer.expect_punctuation("{")?;
            Some(Block::parse(lexer)?)
        } else {
            None
        };
        Ok(If { condition, body, else_body })
    }
}

/// while CONDITION { BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct While {
    pub condition: Box<Expression>,
    pub body: Block,
}

impl While {
    pub fn parse(lexer: &mut Lexer) -> Result<While, ParserError> {
        let condition = Box::new(Expression::parse(lexer)?);
        lexer.expect_punctuation("{")?;
        let body = Block::parse(lexer)?;
        Ok(While { condition, body })
    }
}

/// for VAR in EXPR { BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct For {
    pub var: Identifier,
    pub expr: Box<Expression>,
    pub body: Block,
}

impl For {
    pub fn parse(lexer: &mut Lexer) -> Result<For, ParserError> {
        let var = lexer.expect_identifier()?;
        lexer.expect_keyword(Keyword::In)?;
        let expr = Box::new(Expression::parse(lexer)?);
        lexer.expect_punctuation("{")?;
        let body = Block::parse(lexer)?;
        Ok(For { var, expr, body })
    }
}
