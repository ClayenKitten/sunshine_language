use crate::{
    ast::{
        expression::Block as AstBlock,
        expression::{Expression as AstExpression, Literal},
        statement::LetStatement,
        statement::Statement as AstStatement,
    },
    hir::{
        scope::Scope,
        types::{PrimitiveType, TypeId},
        Block, Expression, ExpressionKind, HirBuilder, Statement, TranslationError,
    },
    lexer::number::Number,
    path::AbsolutePath,
};

use super::PartiallyParsedFunction;

pub(super) struct BodyBuilder<'b> {
    parent: &'b HirBuilder,
    module: AbsolutePath,
    scope: Scope,
}

impl<'b> BodyBuilder<'b> {
    pub fn translate(
        parent: &'b HirBuilder,
        partial: PartiallyParsedFunction,
    ) -> Result<Block, TranslationError> {
        let mut builder = Self {
            parent,
            module: partial.module,
            scope: Scope::new(),
        };

        for (name, type_id) in partial.params {
            builder.scope.insert(name, type_id);
        }

        let body = builder.translate_block(partial.body, false)?;
        if body.type_id() != partial.return_type {
            return Err(TranslationError::TypeMismatch {
                expected: partial.return_type,
                received: body.type_id(),
            });
        }

        Ok(body)
    }

    fn translate_block(
        &mut self,
        block: AstBlock,
        is_loop: bool,
    ) -> Result<Block, TranslationError> {
        if is_loop {
            self.scope = self.scope.child_loop();
        } else {
            self.scope = self.scope.child();
        }
        let block = {
            let mut tail = None;
            let mut statements = Vec::new();
            for stmt in block.statements {
                let stmt = self.translate_stmt(stmt)?;
                statements.push(stmt);
            }
            if let Some(expr) = block.expression {
                let expr = self.translate_expr(*expr)?;
                tail = Some(Box::new(expr));
            }
            Ok(Block(statements, tail))
        };
        self.scope = self.scope.parent().expect("Scope should have parent");
        block
    }

    fn translate_stmt(&mut self, stmt: AstStatement) -> Result<Statement, TranslationError> {
        match stmt {
            AstStatement::ExprStmt(expr) => self.translate_expr(expr).map(Statement::ExprStmt),
            AstStatement::LetStmt(LetStatement { name, type_, value }) => {
                let Some(type_) = type_ else { return Err(TranslationError::TypeInference)};
                let type_ = self.parent.type_table.get(type_)?;
                let value = value
                    .map(|v| *v)
                    .and_then(|expr| self.translate_expr(expr).ok())
                    .map(Box::new);
                let var = self.scope.insert(name, type_);
                Ok(Statement::LetStmt { var, type_, value })
            }
            AstStatement::Assignment {
                assignee,
                operator,
                mut expression,
            } => {
                let Some((var, _)) = self.scope.lookup(&assignee) else {
                    return Err(TranslationError::VariableNotDeclared(assignee))
                };

                if let Some(operator) = operator.to_respective_binary_op() {
                    expression = AstExpression::Binary {
                        op: operator,
                        left: Box::new(AstExpression::Var(assignee)),
                        right: Box::new(expression),
                    };
                }

                Ok(Statement::Assignment {
                    assignee: var,
                    value: self.translate_expr(expression)?,
                })
            }
            AstStatement::Return(expr) => self.translate_expr(expr).map(Statement::Return),
            AstStatement::Break => {
                if self.scope.is_loop() {
                    Ok(Statement::Break)
                } else {
                    Err(TranslationError::InvalidBreak)
                }
            }
        }
    }

    fn translate_expr(&mut self, expr: AstExpression) -> Result<Expression, TranslationError> {
        Ok(match expr {
            AstExpression::Block(block) => {
                let block = self.translate_block(block, false)?;
                Expression {
                    type_: block.type_id(),
                    kind: ExpressionKind::Block(block),
                }
            }
            AstExpression::If {
                condition,
                body,
                else_body,
            } => {
                let condition = self.translate_expr(*condition)?;
                if condition.type_ != Some(TypeId::BOOL) {
                    return Err(TranslationError::TypeMismatch {
                        expected: Some(TypeId::BOOL),
                        received: condition.type_,
                    });
                }

                let body = self.translate_block(body, false)?;
                let else_body = match else_body {
                    Some(else_body) => {
                        let else_body = self.translate_block(else_body, false)?;
                        if body.type_id() != else_body.type_id() {
                            return Err(TranslationError::IfBranchTypeMismatch {
                                body: body.type_id(),
                                else_body: else_body.type_id(),
                            });
                        }
                        Some(else_body)
                    }
                    None => None,
                };

                Expression {
                    type_: body.type_id(),
                    kind: ExpressionKind::If {
                        condition: Box::new(condition),
                        body,
                        else_body,
                    },
                }
            }
            AstExpression::While { condition, body } => {
                let condition = self.translate_expr(*condition)?;
                if condition.type_ != Some(TypeId::BOOL) {
                    return Err(TranslationError::TypeMismatch {
                        expected: Some(TypeId::BOOL),
                        received: condition.type_,
                    });
                }
                let mut body = self.translate_block(body, true)?;
                body.0.insert(
                    0,
                    Statement::ExprStmt(Expression {
                        type_: None,
                        kind: ExpressionKind::If {
                            condition: Box::new(condition),
                            body: Block(vec![Statement::Break], None),
                            else_body: None,
                        },
                    }),
                );
                Expression {
                    type_: None,
                    kind: ExpressionKind::Loop(body),
                }
            }
            AstExpression::For { .. } => todo!(),
            AstExpression::Unary { .. } => todo!(),
            AstExpression::Binary { .. } => todo!(),
            AstExpression::FnCall {
                path,
                params: ast_args,
            } => {
                let path = {
                    let Some(path) = path.to_absolute(&self.module) else {
                        todo!();
                    };
                    path
                };
                let Some((func_id, params, return_type)) = self.parent.query_function_info(&path) else {
                    return Err(TranslationError::FunctionNotFound(path));
                };

                if ast_args.len() != params.len() {
                    return Err(TranslationError::ArgumentCountMismatch {
                        expected: params.len(),
                        received: ast_args.len(),
                    });
                }

                let args = ast_args
                    .into_iter()
                    .zip(params.iter())
                    .map(|(arg, expected)| {
                        let arg = self.translate_expr(arg)?;
                        if arg.type_ != Some(*expected) {
                            return Err(TranslationError::TypeMismatch {
                                expected: Some(*expected),
                                received: arg.type_,
                            });
                        }
                        Ok(arg)
                    })
                    .collect::<Result<_, _>>()?;

                Expression {
                    type_: return_type,
                    kind: ExpressionKind::FnCall(func_id, args),
                }
            }
            AstExpression::Var(var) => match self.scope.lookup(&var) {
                Some((var, type_)) => Expression {
                    type_: Some(type_),
                    kind: ExpressionKind::Var(var),
                },
                None => return Err(TranslationError::VariableNotDeclared(var)),
            },
            AstExpression::Literal(lit) => {
                let type_ = match lit {
                    Literal::Number(Number { fraction: None, .. }) => {
                        TypeId::Primitive(PrimitiveType::I32)
                    }
                    Literal::Number(Number {
                        fraction: Some(_), ..
                    }) => TypeId::Primitive(PrimitiveType::F32),
                    Literal::String(_) => todo!(),
                    Literal::Boolean(_) => TypeId::Primitive(PrimitiveType::Bool),
                };
                Expression {
                    type_: Some(type_),
                    kind: ExpressionKind::Literal(lit),
                }
            }
        })
    }
}
