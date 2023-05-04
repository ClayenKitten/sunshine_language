use crate::{
    ast::{
        expression::{Block, Expression, Literal},
        statement::Statement,
    },
    error::{
        library::{
            lexer::{TokenMismatch, UnexpectedEOF},
            parser::{
                AssignmentInExpressionPosition, InvalidCrateKw, InvalidPunctuation, InvalidSuperKw,
                KeywordNotAllowedInOperatorExpression,
            },
        },
        ExpectedToken, ReportProvider,
    },
    lexer::{keyword::Keyword, punctuation::Punctuation, Token},
    parser::{operator_expression::postfix::PostfixNotation, FileParser, ParserError},
    path::{RelativePath, RelativePathStart},
    Identifier,
};

use super::operator_expression::Tree;

/// [Expression]'s parsing.
///
/// [Expression]: crate::ast::expression::Expression
impl FileParser {
    /// Parse expression.
    pub fn parse_expr(&mut self) -> Result<Expression, ParserError> {
        let start = self.location();
        let infix = self.parse_infix()?;
        let postfix = PostfixNotation::from_infix(infix);
        match postfix.into_expression() {
            Ok(tree) => Ok(tree),
            Err(e) => {
                AssignmentInExpressionPosition::report(self, start);
                Err(e)
            }
        }
    }

    /// Parse a single operand.
    pub(super) fn parse_operand(&mut self) -> Result<Expression, ParserError> {
        use {Keyword::*, Punctuation::*};

        let start = self.location();
        let token = match self.lexer.next()? {
            Token::Punc(LBrace) => Expression::Block(self.parse_block()?),

            Token::Num(num) => Expression::Literal(Literal::Number(num)),
            Token::Str(str) => Expression::Literal(Literal::String(str)),

            Token::Kw(If) => self.parse_if()?,
            Token::Kw(While) => self.parse_while()?,
            Token::Kw(For) => self.parse_for()?,
            Token::Kw(True) => Expression::Literal(Literal::Boolean(true)),
            Token::Kw(False) => Expression::Literal(Literal::Boolean(false)),

            Token::Ident(ident) => {
                let path_start = match ident.as_str() {
                    "super" => RelativePathStart::Super(1),
                    "crate" => RelativePathStart::Crate,
                    _ => RelativePathStart::Identifier(Identifier(ident)),
                };
                let mut path = RelativePath::new(path_start);
                while self.lexer.consume_punctuation("::")? {
                    let ident = self.lexer.expect_identifier()?;
                    match ident.0.as_str() {
                        "super" if !path.other.is_empty() => {
                            InvalidSuperKw::report(self, start);
                            return Err(ParserError::ParserError);
                        }
                        "super" if matches!(path.start, RelativePathStart::Super(_)) => {
                            let RelativePathStart::Super(ref mut n) = path.start else { unreachable!(); };
                            *n += 1;
                        }
                        "crate" => {
                            InvalidCrateKw::report(self, start);
                            return Err(ParserError::ParserError);
                        }
                        _ => path.push(ident),
                    };
                }

                if self.lexer.consume_punctuation("(")? {
                    let mut params = Vec::new();
                    if self.lexer.consume_punctuation(")")? {
                        return Ok(Expression::FnCall { path, params });
                    }

                    return loop {
                        let start = self.location();
                        params.push(self.parse_expr()?);

                        if self.lexer.consume_punctuation(")")? {
                            break Ok(Expression::FnCall { path, params });
                        }

                        if !self.lexer.consume_punctuation(",")? {
                            let token = self.lexer.peek()?;
                            TokenMismatch::report(
                                self,
                                start,
                                vec![
                                    ExpectedToken::Punctuation(Punctuation::Comma),
                                    ExpectedToken::Punctuation(Punctuation::RParent),
                                ],
                                token,
                            );
                            break Err(ParserError::ParserError);
                        }
                    };
                }

                match path {
                    RelativePath {
                        start: RelativePathStart::Identifier(ident),
                        other,
                    } if other.is_empty() => Expression::Var(ident),
                    _ => todo!(),
                }
            }

            Token::Eof => {
                UnexpectedEOF::report(self, start);
                return Err(ParserError::ParserError);
            }

            Token::Kw(kw) => {
                KeywordNotAllowedInOperatorExpression::report(self, start, kw);
                return Err(ParserError::ParserError);
            }

            Token::Punc(punc) => {
                InvalidPunctuation::report(self, start, punc);
                return Err(ParserError::ParserError);
            }
        };
        Ok(token)
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
