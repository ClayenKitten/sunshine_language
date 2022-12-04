mod shunting_yard;

use crate::lexer::{number::Number, Token, Lexer, punctuation::Punctuation, keyword::Keyword};

use self::shunting_yard::ReversePolishNotation;

use super::{ParserError, UnexpectedTokenError, Statement, statement::Block};

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Block),
    
    /// Expression with operators stored in reverse polish notation.
    Polish(ReversePolishNotation),
    
    If(If),
    While(While),
    For(For),

    Identifier(Identifier),
    Literal(Literal),
    Assignment(Assignment),
    
    FunctionCall(FunctionCall),
    Variable(Identifier),
}

impl Expression {
    pub fn parse(lexer: &mut Lexer) -> Result<Expression, ParserError> {
        shunting_yard::ReversePolishNotation::parse(lexer)
            .map(|expr| Self::Polish(expr))
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
    fn maybe_function_call(lexer: &mut Lexer, name: Identifier) -> Result<Expression, ParserError> {
        if lexer.peek()? == Token::Punctuation(Punctuation::new("(")) {
            let mut params = Vec::new();
            loop {
                let expr = Expression::parse(lexer)?;
                params.push(expr);

                if ")" == lexer.expect_punctuation([")", ","])? {
                    return Ok(Expression::FunctionCall(FunctionCall { name, params }));
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
        match self {
            Expression::Block(_) => true,
            Expression::If(_) => true,
            Expression::While(_) => true,
            Expression::For(_) => true,
            _ => false,
        }
    }
}

/// Identifier is name of type, variable or function.
#[derive(Debug, PartialEq, Eq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn parse(lexer: &mut Lexer) -> Result<Identifier, ParserError> {
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
    pub body: Block,
    pub else_body: Option<Block>,
}

impl If {
    pub fn parse(lexer: &mut Lexer) -> Result<If, ParserError> {
        let condition = Box::new(Expression::parse(lexer)?);
        lexer.expect_punctuation(["{"])?;
        let body = Block::parse(lexer)?;
        let else_body = if let Token::Keyword(Keyword::Else) = lexer.peek()? {
            lexer.expect_punctuation(["{"])?;
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
    pub body: Vec<Statement>,
}

/// for VAR in EXPR { BODY }
#[derive(Debug, PartialEq, Eq)]
pub struct For {
    pub var: Identifier,
    pub expr: Box<Expression>,
    pub body: Vec<Statement>,
}
