mod operator;

pub use operator::{UnaryOp, BinaryOp};

use crate::lexer::{number::Number, Token, TokenStream, punctuation::Punctuation, keyword::Keyword};

use super::{ParserError, UnexpectedTokenError, Statement};

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Vec<Statement>),
    /// Group is a single parenthized expression.
    Group(Box<Expression>),

    Unary(UnaryOp),
    Binary(BinaryOp),
    
    If(If),
    While(While),
    For(For),

    Identifier(Identifier),
    Literal(Literal),
    Assignment(Assignment),
    FunctionCall(FunctionCall),
}

impl Expression {
    pub fn parse(lexer: &mut TokenStream) -> Result<Expression, ParserError> {
        Ok(match lexer.next_some()? {
            Token::Punctuation(Punctuation("{")) => {
                Expression::Block(Statement::parse_block(lexer)?)
            }

            Token::Punctuation(Punctuation("(")) => {
                Expression::parse_delimited_parenthesis(lexer)?
            }

            Token::Punctuation(punc) => {
                if punc.is_unary_operator() {
                    let operand = Expression::parse(lexer)?;
                    Expression::Unary(UnaryOp {
                        operator: punc,
                        operand: Box::new(operand),
                    })
                } else {
                    return Err(UnexpectedTokenError::UnexpectedToken(Token::Punctuation(punc)).into())
                }
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
                    _ => unreachable!(),
                }
            }
            
            Token::Identifier(ident) => {
                Self::parse_combined(lexer)?.0
            },

            Token::Eof => unreachable!(),
        })
    }

    /// Parse non-primitive expression that combines function calls, paths, binary operators, etc.
    /// 
    /// Parsing continues until "stopper" punctuation met or error occur.
    /// 
    /// # Returns
    /// 
    /// Both parsed expression and stopper are returned.
    fn parse_combined(lexer: &mut TokenStream) -> Result<(Expression, Punctuation), ParserError> {
        enum StackEntry {
            Operator(Punctuation),
            Operand(Expression),
        }

        let mut stack = Vec::<StackEntry>::new();

        todo!();
    }

    /// Parse expressions delimited by commas (`,`) until closing parenthesis (`)`) met.
    fn parse_delimited_parenthesis(lexer: &mut TokenStream) -> Result<Expression, ParserError> {
        todo!()
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
