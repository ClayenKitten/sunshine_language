use std::collections::HashMap;

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

        let mut parameters = HashMap::with_capacity(params.len());
        for Parameter { name, type_ } in params {
            parameters.insert(name, self.parent.type_table.get(type_)?);
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
            body: self.translate_block(body)?,
        })
    }

    fn translate_block(&mut self, block: AstBlock) -> Result<Block, TranslationError> {
        self.scope = self.scope.child();
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
                Ok(Statement::LetStmt { name, type_, value })
            }
            AstStatement::Assignment { .. } => todo!(),
            AstStatement::Return(expr) => self.translate_expr(expr).map(Statement::Return),
            AstStatement::Break => todo!(),
        }
    }

    fn translate_expr(&mut self, expr: AstExpression) -> Result<Expression, TranslationError> {
        Ok(match expr {
            AstExpression::Block(block) => Expression::Block(self.translate_block(block)?),
            AstExpression::If { .. } => todo!(),
            AstExpression::While { .. } => todo!(),
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
                    return Err(TranslationError::FunctionNotFound(path.clone()));
                };
                Expression::FnCall(id, params)
            }
            AstExpression::Var(_) => todo!(),
            AstExpression::Literal(lit) => Expression::Literal(lit),
        })
    }
}
