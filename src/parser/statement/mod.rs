mod r#let;

use crate::lexer::{Lexer, Token, punctuation::Punctuation, keyword::Keyword};

use self::r#let::LetStatement;

use super::{item::Item, Expression, ParserError, UnexpectedTokenError, expressions::Identifier};

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Item(Item),
    ExpressionStatement(Expression),
    LetStatement(LetStatement),
    Assignment {
        op: Punctuation,
        left: Identifier,
        right: Box<Expression>,
    },
}

/// Block is an expression that consists of a number of statements and an optional final expression.
#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    statements: Vec<Statement>,
    expression: Option<Box<Expression>>,
}

impl Block {
    pub fn parse(lexer: &mut Lexer) -> Result<Block, ParserError> {
        let mut buffer = Vec::new();
        let expr = loop {
            let statement = match lexer.peek()? {
                Token::Punctuation(Punctuation("}"))
                    => { let _ = lexer.next(); break None; },
                Token::Keyword(Keyword::Fn | Keyword::Struct)
                    => Statement::Item(Item::parse(lexer)?),
                Token::Keyword(Keyword::Let)
                    => Statement::LetStatement(LetStatement::parse(lexer)?),
                Token::Eof
                    => return Err(ParserError::UnexpectedEof),
                _ => {
                    let expr = Expression::parse(lexer)?;
                    if lexer.consume_punctuation("}")? {
                        break Some(expr);
                    }                    
                    if expr.is_block_expression() {
                        lexer.consume_punctuation(";")?;
                    } else {
                        lexer.expect_punctuation([";"])?;
                    }
                    Statement::ExpressionStatement(expr)
                },
            };
            buffer.push(statement);
        };
        Ok(Block { statements: buffer, expression: expr.map(Box::new) })
    }
}
