use crate::{lexer::{number::Number, Token, Lexer, punctuation::Punctuation, keyword::Keyword}, parser::{ParserError, UnexpectedTokenError}};

use super::{statement::Block};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    #[deprecated = "use `Parser::parse_expr`"]
    pub fn parse(lexer: &mut Lexer) -> Result<Expression, ParserError> {
        panic!("To be removed");
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Number(Number),
    String(String),
    Boolean(bool),
}

/// NAME(PARAMS, ...)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub params: Vec<Expression>,
}

/// if CONDITION { BODY } else { ELSE_BODY }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub condition: Box<Expression>,
    pub body: Block,
    pub else_body: Option<Block>,
}

impl If {
    #[deprecated = "use `Parser::parse_if`"]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct While {
    pub condition: Box<Expression>,
    pub body: Block,
}

impl While {
    #[deprecated = "use `Parser::parse_while`"]
    pub fn parse(lexer: &mut Lexer) -> Result<While, ParserError> {
        let condition = Box::new(Expression::parse(lexer)?);
        lexer.expect_punctuation("{")?;
        let body = Block::parse(lexer)?;
        Ok(While { condition, body })
    }
}

/// for VAR in EXPR { BODY }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct For {
    pub var: Identifier,
    pub expr: Box<Expression>,
    pub body: Block,
}

impl For {
    #[deprecated = "use `Parser::parse_for`"]
    pub fn parse(lexer: &mut Lexer) -> Result<For, ParserError> {
        let var = lexer.expect_identifier()?;
        lexer.expect_keyword(Keyword::In)?;
        let expr = Box::new(Expression::parse(lexer)?);
        lexer.expect_punctuation("{")?;
        let body = Block::parse(lexer)?;
        Ok(For { var, expr, body })
    }
}
