pub mod r#let;

use crate::{lexer::{Lexer, Token, punctuation::Punctuation, keyword::Keyword}, parser::ParserError};

use self::r#let::LetStatement;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    ExpressionStatement(Expression),
    LetStatement(LetStatement),
    Break,
}

/// Block is an expression that consists of a number of statements and an optional final expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}

impl Block {
    #[deprecated = "use `Parser::parse_block`"]
    pub fn parse(lexer: &mut Lexer) -> Result<Block, ParserError> {
        let mut buffer = Vec::new();
        let expr = loop {
            let statement = match lexer.peek()? {
                Token::Punctuation(Punctuation("}")) => {
                    let _ = lexer.next();
                    break None;
                },
                Token::Keyword(Keyword::Fn | Keyword::Struct)
                    => panic!("To be removed"),
                Token::Keyword(Keyword::Let)
                    => Statement::LetStatement(LetStatement::parse(lexer)?),
                Token::Keyword(Keyword::Break) => {
                    let _ = lexer.next();
                    lexer.expect_punctuation(";")?;
                    Statement::Break
                },
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
                        lexer.expect_punctuation(";")?;
                    }
                    Statement::ExpressionStatement(expr)
                },
            };
            buffer.push(statement);
        };
        Ok(Block { statements: buffer, expression: expr.map(Box::new) })
    }
}
