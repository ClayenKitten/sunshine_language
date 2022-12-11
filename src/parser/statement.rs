use crate::{ast::{statement::{Block, Statement, r#let::LetStatement}}, lexer::{keyword::Keyword}};

use super::{Parser, ParserError};

impl<'s> Parser<'s> {
    pub fn parse_block(&mut self) -> Result<Block, ParserError> {
        let mut buffer = Vec::new();
        let expr = loop {
            if self.lexer.consume_punctuation("}")? {
                break None;
            }

            if self.lexer.consume_keyword(Keyword::Fn)? || self.lexer.consume_keyword(Keyword::Struct)? {
                self.parse_item()?;
                continue;
            }

            if self.lexer.consume_keyword(Keyword::Let)? {
                buffer.push(Statement::LetStatement(LetStatement::parse(&mut self.lexer)?));
                continue;
            }
            
            if self.lexer.consume_keyword(Keyword::Break)? {
                self.lexer.expect_punctuation(";")?;
                buffer.push(Statement::Break);
                continue;
            }

            let expr = self.parse_expr()?;
            if self.lexer.consume_punctuation("}")? {
                break Some(expr);
            }
            if expr.is_block_expression() {
                self.lexer.consume_punctuation(";")?;
            } else {
                self.lexer.expect_punctuation(";")?;
            }
            buffer.push(Statement::ExpressionStatement(expr));
        };
        Ok(Block { statements: buffer, expression: expr.map(Box::new) })
    }
}
