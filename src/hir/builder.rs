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
    path::ItemPath,
    item_table::ItemTable,
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
    current_function: Option<ItemPath>,
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
        let mut functions: Vec<(ItemPath, AstFunction)> = Vec::new();

        for (path, item) in item_table.into_iter() {
            match item.kind {
                ItemKind::Module(_) => {}
                ItemKind::Struct(strukt) => {
                    let id = self.type_table.define_name(strukt.name.clone());
                    strukts.push((id, strukt.fields));
                }
                ItemKind::Function(function) => {
                    let id = FunctionId(self.function_mapping.len() as u32);
                    self.function_mapping.insert(path.clone(), id);
                    functions.push((path, function));
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

        for (path, function) in functions {
            self.current_function = Some(path);
            let function = self.translate_func(function);
            match function {
                Ok(function) => self.functions.push(function),
                Err(error) => self.errors.push(error),
            }
            self.current_function = None;
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    fn translate_func(&mut self, func: AstFunction) -> Result<Function, TranslationError> {
        let AstFunction {
            name: _,
            params,
            return_type,
            body,
        } = func;
        Ok(Function {
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
                
                let mut context_path = self.current_function.clone()
                    .expect("`current_function` should be set");
                context_path.pop();
                for segment in path {
                    context_path.push(segment);
                }
                let Some(id) = self.function_mapping.get(&context_path).copied() else {
                    return Err(TranslationError::FunctionNotFound(context_path.clone()));
                };
                Expression::FnCall(id, params)
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
    #[error("function {0} is not found")]
    FunctionNotFound(ItemPath),
    #[error(transparent)]
    TypeError(#[from] TypeError),
}
