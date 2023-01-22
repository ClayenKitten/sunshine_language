use crate::{
    ast::{
        expression::{Block, Expression, Literal},
        statement::Statement,
        Identifier,
    },
    lexer::{keyword::Keyword, punctuation::Punctuation, Token},
    parser::{
        operator_expression::{InfixExpr, ReversePolishExpr},
        FileParser, ParserError, UnexpectedTokenError,
    },
};

/// [Expression]'s parsing.
///
/// [Expression]: crate::ast::expression::Expression
impl FileParser {
    /// Parse expression.
    pub fn parse_expr(&mut self) -> Result<Expression, ParserError> {
        let infix = InfixExpr::parse(self)?;
        let polish = Into::<ReversePolishExpr>::into(infix);
        let tree = polish.into_tree();
        Ok(tree)
    }

    /// Parse a single operand.
    pub(super) fn parse_operand(&mut self) -> Result<Expression, ParserError> {
        use Keyword::*;
        Ok(match self.lexer.next()? {
            Token::Punc(Punctuation("{")) => Expression::Block(self.parse_block()?),

            Token::Num(num) => Expression::Literal(Literal::Number(num)),
            Token::Str(str) => Expression::Literal(Literal::String(str)),

            Token::Kw(If) => self.parse_if()?,
            Token::Kw(While) => self.parse_while()?,
            Token::Kw(For) => self.parse_for()?,
            Token::Kw(True) => Expression::Literal(Literal::Boolean(true)),
            Token::Kw(False) => Expression::Literal(Literal::Boolean(false)),

            Token::Ident(ident) => self.maybe_function_call(Identifier(ident))?,

            Token::Eof => return Err(ParserError::UnexpectedEof),

            Token::Punc(_) | Token::Kw(_) => return Err(UnexpectedTokenError::TokenMismatch.into()),
        })
    }

    /// Try to wrap provided identifier in function call.
    fn maybe_function_call(&mut self, name: Identifier) -> Result<Expression, ParserError> {
        if self.lexer.consume_punctuation("(")? {
            let mut params = Vec::new();
            loop {
                params.push(self.parse_expr()?);
                if self.lexer.consume_punctuation(")")? {
                    return Ok(Expression::FnCall { name, params });
                } else if self.lexer.consume_punctuation(",")? {
                } else {
                    return Err(UnexpectedTokenError::TokenMismatch.into());
                }
            }
        } else {
            Ok(Expression::Var(name))
        }
    }

    /// Parse block. Opening brace is expected to be consumed beforehand.
    pub fn parse_block(&mut self) -> Result<Block, ParserError> {
        let mut buffer = Vec::new();
        let expr = loop {
            if self.lexer.consume_punctuation("}")? {
                break None;
            }

            if self.lexer.consume_keyword(Keyword::Fn)?
                || self.lexer.consume_keyword(Keyword::Struct)?
            {
                self.parse_item()?;
                continue;
            }

            if self.lexer.consume_keyword(Keyword::Return)? {
                buffer.push(Statement::Return(self.parse_expr()?));
                self.lexer.expect_punctuation(";")?;
                continue;
            }

            if self.lexer.consume_keyword(Keyword::Let)? {
                buffer.push(Statement::LetStmt(self.parse_let()?));
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
            buffer.push(Statement::ExprStmt(expr));
        };
        Ok(Block {
            statements: buffer,
            expression: expr.map(Box::new),
        })
    }

    /// Parse if conditional. Keyword [if](Keyword::If) is expected to be consumed beforehand.
    pub fn parse_if(&mut self) -> Result<Expression, ParserError> {
        let condition = Box::new(self.parse_expr()?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;

        let else_body = if self.lexer.consume_keyword(Keyword::Else)? {
            self.lexer.expect_punctuation("{")?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Expression::If {
            condition,
            body,
            else_body,
        })
    }

    /// Parse while loop. Keyword [while](Keyword::While) is expected to be consumed beforehand.
    pub fn parse_while(&mut self) -> Result<Expression, ParserError> {
        let condition = Box::new(self.parse_expr()?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;
        Ok(Expression::While { condition, body })
    }

    /// Parse for loop. Keyword [for](Keyword::For) is expected to be consumed beforehand.
    pub fn parse_for(&mut self) -> Result<Expression, ParserError> {
        let var = self.lexer.expect_identifier()?;
        self.lexer.expect_keyword(Keyword::In)?;
        let expr = Box::new(self.parse_expr()?);
        self.lexer.expect_punctuation("{")?;
        let body = self.parse_block()?;
        Ok(Expression::For { var, expr, body })
    }
}
