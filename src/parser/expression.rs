use crate::{
    ast::{
        expression::{Block, Expression, Literal},
        statement::Statement,
        Identifier,
    },
    error::{
        library::parser::{
            InvalidPunctuation, KeywordNotAllowedInOperatorExpression,
            UnexpectedTokenInFunctionCall,
        },
        ReportProvider,
    },
    lexer::{keyword::Keyword, punctuation::Punctuation, Token},
    parser::{operator_expression::postfix::PostfixNotation, FileParser, ParserError},
};

use super::operator_expression::Tree;

/// [Expression]'s parsing.
///
/// [Expression]: crate::ast::expression::Expression
impl FileParser {
    /// Parse expression.
    pub fn parse_expr(&mut self) -> Result<Expression, ParserError> {
        let infix = self.parse_infix()?;
        let postfix = PostfixNotation::from_infix(infix);
        let tree = postfix.into_expression()?;
        Ok(tree)
    }

    /// Parse a single operand.
    pub(super) fn parse_operand(&mut self) -> Result<Expression, ParserError> {
        use Keyword::*;
        let start = self.location();
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

            Token::Kw(kw) => {
                KeywordNotAllowedInOperatorExpression::report(self, start, kw);
                return Err(ParserError::Obsolete);
            }
            Token::Punc(punc) => {
                InvalidPunctuation::report(self, start, punc);
                return Err(ParserError::Obsolete);
            }
        })
    }

    /// Try to wrap provided identifier in function call.
    fn maybe_function_call(&mut self, name: Identifier) -> Result<Expression, ParserError> {
        if self.lexer.consume_punctuation("(")? {
            let mut params = Vec::new();
            loop {
                let start = self.location();
                params.push(self.parse_expr()?);
                if self.lexer.consume_punctuation(")")? {
                    return Ok(Expression::FnCall { name, params });
                } else if self.lexer.consume_punctuation(",")? {
                } else {
                    let token = self.lexer.peek()?;
                    UnexpectedTokenInFunctionCall::report(self, start, token);
                    return Err(ParserError::Obsolete);
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

            let infix = self.parse_infix()?;
            let postfix = PostfixNotation::from_infix(infix);
            let tree = postfix.into_tree();
            match tree {
                Tree::Assignment {
                    assignee,
                    operator,
                    expression,
                } => buffer.push(Statement::Assignment {
                    assignee,
                    operator,
                    expression,
                }),
                Tree::Expression(expr) => {
                    if self.lexer.consume_punctuation("}")? {
                        break Some(expr);
                    }
                    if expr.is_block_expression() {
                        self.lexer.consume_punctuation(";")?;
                    } else {
                        self.lexer.expect_punctuation(";")?;
                    }
                    buffer.push(Statement::ExprStmt(expr));
                }
            }
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
