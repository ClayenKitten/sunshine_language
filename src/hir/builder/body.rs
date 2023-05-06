use crate::{
    ast::{
        expression::Block as AstBlock, expression::Expression as AstExpression,
        item::Function as AstFunction, item::Parameter, statement::LetStatement,
        statement::Statement as AstStatement,
    },
    hir::{
        scope::Scope, Block, Expression, Function, FunctionSignature, HirBuilder, Statement,
        TranslationError,
    },
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

        Ok(Function {
            signature,
            body: self.translate_block(body, false)?,
        })
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
            let mut result = Vec::new();
            for stmt in block.statements {
                let stmt = self.translate_stmt(stmt)?;
                result.push(stmt);
            }
            if let Some(expr) = block.expression {
                let expr = self.translate_expr(*expr)?;
                result.push(Statement::Return(expr))
            }
            Ok(Block(result))
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
            AstExpression::Block(block) => Expression::Block(self.translate_block(block, false)?),
            AstExpression::If {
                condition,
                body,
                else_body,
            } => Expression::If {
                condition: Box::new(self.translate_expr(*condition)?),
                body: self.translate_block(body, false)?,
                else_body: match else_body {
                    Some(else_body) => Some(self.translate_block(else_body, false)?),
                    None => None,
                },
            },
            AstExpression::While { condition, body } => {
                let condition = self.translate_expr(*condition)?;
                let mut body = self.translate_block(body, true)?;
                body.0.insert(
                    0,
                    Statement::ExprStmt(Expression::If {
                        condition: Box::new(condition),
                        body: Block(vec![Statement::Break]),
                        else_body: None,
                    }),
                );
                Expression::Loop(body)
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
                Expression::FnCall(id, params)
            }
            AstExpression::Var(var) => match self.scope.lookup(&var) {
                Some((var, _)) => Expression::Var(var),
                None => return Err(TranslationError::VariableNotDeclared(var)),
            },
            AstExpression::Literal(lit) => Expression::Literal(lit),
        })
    }
}
