use crate::{
    ast::{expressions::{Expression, For, If, While, Literal, Identifier, FunctionCall, Block}, statement::Statement},
    lexer::{keyword::Keyword, Token, punctuation::Punctuation}, parser::UnexpectedTokenError,
};

use super::{Parser, ParserError, shunting_yard};

impl<'s> Parser<'s> {
    pub fn parse_expr(&mut self) -> Result<Expression, ParserError> {
        shunting_yard::ReversePolishNotation::parse(self)
            .map(|expr| expr.into_tree())
    }

    /// Parse a single operand
    pub(super) fn parse_operand(&mut self) -> Result<Expression, ParserError> {
        let token = match self.lexer.next()? {
            Token::Punctuation(Punctuation("{")) => {
                Expression::Block(self.parse_block()?)
            }

            Token::Punctuation(_) => {
                return Err(UnexpectedTokenError::TokenMismatch.into())
            }

            Token::Number(num) => Expression::Literal(Literal::Number(num)),
            Token::String(str) => Expression::Literal(Literal::String(str)),

            Token::Keyword(kw) => {
                match kw {
                    Keyword::If => Expression::If(self.parse_if()?),
                    Keyword::While => Expression::While(self.parse_while()?),
                    Keyword::For => Expression::For(self.parse_for()?),
                    Keyword::True => Expression::Literal(Literal::Boolean(true)),
                    Keyword::False => Expression::Literal(Literal::Boolean(false)),
                    _ => return Err(UnexpectedTokenError::TokenMismatch.into()),
                }
            }

            Token::Identifier(ident) => {
                self.maybe_function_call(Identifier(ident))?
            },

            Token::Eof => return Err(ParserError::UnexpectedEof),
        };
        Ok(token)
    }

    /// Try to wrap provided identifier in function call.
    fn maybe_function_call(&mut self, name: Identifier) -> Result<Expression, ParserError> {
        if self.lexer.consume_punctuation("(")? {
            let mut params = Vec::new();
            loop {
                params.push(self.parse_expr()?);
                if self.lexer.consume_punctuation(")")? {
                    return Ok(Expression::FunctionCall(FunctionCall { name, params }));
                } else if self.lexer.consume_punctuation(",")? {
                    
                } else {
                    return Err(UnexpectedTokenError::TokenMismatch.into());
                }
            }
        } else {
            Ok(Expression::Variable(name))
        }
    }

    /// Parse block. Opening brace is expected to be consumed beforehand.
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

    /// Parse if loop. Keyword `if` is expected to be consumed beforehand.
    pub fn parse_if(&mut self) -> Result<If, ParserError> {
        let condition = Box::new(self.parse_expr()?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;

        let else_body = if self.lexer.consume_keyword(Keyword::Else)? {
            self.lexer.expect_punctuation("{")?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(If {
            condition,
            body,
            else_body,
        })
    }

    /// Parse while loop. Keyword `while` is expected to be consumed beforehand.
    pub fn parse_while(&mut self) -> Result<While, ParserError> {
        let condition = Box::new(self.parse_expr()?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;
        Ok(While { condition, body })
    }

    /// Parse for loop. Keyword `for` is expected to be consumed beforehand.
    pub fn parse_for(&mut self) -> Result<For, ParserError> {
        let var = self.lexer.expect_identifier()?;
        self.lexer.expect_keyword(Keyword::In)?;
        let expr = Box::new(self.parse_expr()?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;
        Ok(For { var, expr, body })
    }
}
