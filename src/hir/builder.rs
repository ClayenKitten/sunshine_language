use std::collections::HashMap;

use crate::{
    ast::{
        expression::Block as AstBlock,
        expression::Expression as AstExpression,
        item::Function as AstFunction,
        item::{ItemKind, Parameter, Field},
        statement::LetStatement,
        statement::Statement as AstStatement,
    },
    hir,
    item_table::{path::ItemPath, ItemTable},
};

use super::{
    scope::Scope,
    types::{TypeError, TypeTable, TypeId},
    Expression, Function, FunctionId, Hir, Statement, Block,
};

use thiserror::Error;

#[derive(Debug, Default)]
pub struct HirBuilder {
    type_table: TypeTable,
    function_mapping: HashMap<ItemPath, FunctionId>,
    functions: Vec<Function>,
    errors: Vec<TranslationError>,
    scope: Scope,
}

impl HirBuilder {
    pub fn build(self) -> Result<Hir, Vec<TranslationError>> {
        if self.errors.is_empty() {
            Ok(Hir {
                type_table: self.type_table,
                functions: self.functions,
            })
        } else {
            Err(self.errors)
        }
    }

    pub fn populate(&mut self, item_table: ItemTable) {
        let mut strukts: Vec<(TypeId, Vec<Field>)> = Vec::new();

        for (path, item) in item_table.into_iter() {
            match item.kind {
                ItemKind::Module(_) => {}
                ItemKind::Struct(strukt) => {
                    let id = self.type_table.define_name(strukt.name.clone());
                    strukts.push((id, strukt.fields));
                }
                ItemKind::Function(function) => {
                    let function = self.translate_func(function);
                    match function {
                        Ok(function) => {
                            let id = FunctionId(self.functions.len() as u32);
                            self.functions.push(function);
                            self.function_mapping.insert(path, id);
                        },
                        Err(error) => self.errors.push(error),
                    }
                }
            }
        }

        for (id, fields) in strukts {
            for Field { name, type_ } in fields {
                let result = self.type_table.add_field(id, name, type_);
                if let Err(err) = result {
                    self.errors.push(err.into());
                }
            }
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    fn translate_func(&mut self, func: AstFunction) -> Result<Function, TranslationError> {
        let AstFunction {
            name,
            params,
            return_type,
            body,
        } = func;
        Ok(Function {
            name,
            params: params
                .into_iter()
                .map(|Parameter { name, type_ }| Ok::<_, TypeError>((name, self.type_table.get(type_)?)))
                .collect::<Result<_, _>>()?,
            return_type: match return_type {
                Some(type_) => Some(self.type_table.get(type_)?),
                None => None,
            },
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
                result.push(hir::Statement::Return(expr))
            }
            Ok(hir::Block(result))
        };
        self.scope = self.scope.parent().unwrap_or_default();
        block
    }

    fn translate_stmt(&mut self, stmt: AstStatement) -> Result<Statement, TranslationError> {
        match stmt {
            AstStatement::ExprStmt(expr) => {
                self.translate_expr(expr).map(Statement::ExprStmt)
            }
            AstStatement::LetStmt(LetStatement { name, type_, value }) => {
                let Some(type_) = type_ else { return Err(TranslationError::TypeInference)};
                let type_ = self.type_table.get(type_)?;
                let value = value
                    .map(|v| *v)
                    .and_then(|expr| self.translate_expr(expr).ok())
                    .map(Box::new);
                Ok(hir::Statement::LetStmt { name, type_, value })
            }
            AstStatement::Assignment {
                assignee,
                operator,
                expression,
            } => todo!(),
            AstStatement::Return(_) => todo!(),
            AstStatement::Break => todo!(),
        }
    }

    fn translate_expr(&mut self, expr: AstExpression) -> Result<Expression, TranslationError> {
        Ok(match expr {
            AstExpression::Block(block) => {
                hir::Expression::Block(self.translate_block(block)?)
            }
            AstExpression::If {condition, body, else_body} => todo!(),
            AstExpression::While { condition, body } => todo!(),
            AstExpression::For { var, expr, body } => todo!(),
            AstExpression::Unary { op, value } => todo!(),
            AstExpression::Binary { op, left, right } => todo!(),
            AstExpression::FnCall { path, params: ast_params } => {
                let mut params = Vec::with_capacity(ast_params.capacity());
                for param in ast_params {
                    params.push(self.translate_expr(param)?);
                }
                Expression::FnCall(path, params)
            }
            AstExpression::Var(name) => todo!(),
            AstExpression::Literal(lit) => Expression::Literal(lit),
        })
    }
}

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("type inference is not implemented yet, so type annotation is required for every variable binding")]
    TypeInference,
    #[error(transparent)]
    TypeError(#[from] TypeError),
}
