use crate::{
    ast::statement::Statement,
    lexer::{
        number::Number,
        operator::{BinaryOp, UnaryOp},
    },
    Identifier
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// Block is a set of statements surrounded by opening and closing brace.
    Block(Block),

    If {
        condition: Box<Expression>,
        body: Block,
        else_body: Option<Block>,
    },
    While {
        condition: Box<Expression>,
        body: Block,
    },
    For {
        var: Identifier,
        expr: Box<Expression>,
        body: Block,
    },

    Unary {
        op: UnaryOp,
        value: Box<Expression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    FnCall {
        path: Vec<Identifier>,
        params: Vec<Expression>,
    },
    Var(Identifier),
    Literal(Literal),
}

impl Expression {
    /// Check if that expression is block expression.
    ///
    /// Block expressions end with a right brace and don't require to be followed by a semicolon to
    /// be accounted as expression statement.
    pub fn is_block_expression(&self) -> bool {
        matches!(
            self,
            Expression::Block(_)
                | Expression::If { .. }
                | Expression::While { .. }
                | Expression::For { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Number(Number),
    String(String),
    Boolean(bool),
}

/// Block is an expression that consists of a number of statements and an optional final expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}
