use crate::{
    ast::{
        expression::Block as AstBlock,
        expression::{Expression as AstExpression, Literal},
        item::Function as AstFunction,
        item::Parameter,
        statement::LetStatement,
        statement::Statement as AstStatement,
    },
    hir::{
        scope::Scope,
        types::{PrimitiveType, TypeId},
        Block, Expression, ExpressionKind, Function, FunctionSignature, HirBuilder, Statement,
        TranslationError,
    },
    lexer::number::Number,
    path::AbsolutePath,
};

pub(super) struct BodyBuilder<'b> {
    parent: &'b HirBuilder,
    module: AbsolutePath,
    scope: Scope,
}

impl<'b> BodyBuilder<'b> {
    pub fn translate(
        parent: &'b HirBuilder,
        module: AbsolutePath,
        function: AstFunction,
    ) -> Result<Function, TranslationError> {
        let mut builder = Self {
            parent,
            module,
            scope: Scope::new(),
        };
        builder.translate_function(function)
    }

    fn translate_function(&mut self, func: AstFunction) -> Result<Function, TranslationError> {
        let AstFunction {
            name: _,
            params,
            return_type,
            body,
        } = func;

        let mut parameters = Vec::with_capacity(params.len());
        for Parameter { name, type_ } in params {
            let type_id = self.parent.type_table.get(type_)?;
            let var_id = self.scope.insert(name, type_id);
            parameters.push((var_id, type_id));
        }

        let signature = FunctionSignature {
            params: parameters,
            return_type: match return_type {
                Some(type_) => Some(self.parent.type_table.get(type_)?),
                None => None,
            },
        };

        let body = self.translate_block(body, false)?;
        if body.type_id() != signature.return_type {
            return Err(TranslationError::TypeMismatch {
                expected: signature.return_type,
                received: body.type_id(),
            });
        }

        Ok(Function { signature, body })
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
                let else_body = else_body
                    .map(|body| self.translate_block(body, false))
                    .transpose()?;
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
                params: ast_params,
            } => {
                let mut params = Vec::with_capacity(ast_params.capacity());
                for param in ast_params {
                    params.push(self.translate_expr(param)?);
                }

                let path = {
                    let Some(path) = path.to_absolute(&self.module) else {
                        todo!();
                    };
                    path
                };

                let Some(id) = self.parent.function_mapping.get(&path).copied() else {
                    return Err(TranslationError::FunctionNotFound(path));
                };
                let type_ = self.parent.functions[id.0 as usize].signature.return_type;
                Expression {
                    type_,
                    kind: ExpressionKind::FnCall(id, params),
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
