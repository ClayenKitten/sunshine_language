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
                buffer.push(Statement::LetStatement(self.parse_let()?));
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

    /// Parse let statement. `let` keyword is expected to be consumed beforehand.
    pub fn parse_let(&mut self) -> Result<LetStatement, ParserError> {
        self.lexer.expect_keyword(Keyword::Let)?;
        let name = self.lexer.expect_identifier()?;
        let mut statement = LetStatement {
            name,
            type_: None,
            value: None,
        };
        if self.lexer.consume_punctuation(":")? {
            statement.type_ = Some(self.lexer.expect_identifier()?);
        }
        if self.lexer.consume_punctuation("=")? {
            statement.value = Some(Box::new(self.parse_expr()?));
        }
        self.lexer.expect_punctuation(";")?;
        Ok(statement)
    }
}
