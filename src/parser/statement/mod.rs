mod r#let;

use crate::lexer::{TokenStream, Token, punctuation::Punctuation, keyword::Keyword};

use self::r#let::LetStatement;

use super::{item::Item, Expression, ParserError, Delimiter};

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Item(Item),
    ExpressionStatement(Expression),
    LetStatement(LetStatement),
}

impl Statement {
    /// Parse statements until closing delimiter met.
    pub fn parse_block(lexer: &mut TokenStream) -> Result<Vec<Statement>, ParserError> {
        let mut buffer = Vec::new();
        loop {
            let token = lexer.peek()?;
            let statement = match token {
                Token::Punctuation(Punctuation("}"))
                    => { let _ = lexer.next(); break; },
                Token::Keyword(Keyword::Fn | Keyword::Struct)
                    => Statement::Item(Item::parse(lexer)?),
                Token::Keyword(Keyword::Let)
                    => Statement::LetStatement(LetStatement::parse(lexer)?),
                Token::Eof
                    => return Err(ParserError::UnexpectedEof),
                _ => {
                    let expr = Expression::parse(lexer)?;
                    Statement::ExpressionStatement(expr)
                },
            };
            buffer.push(statement);
        }
        Ok(buffer)
    }
}
