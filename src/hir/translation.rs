use crate::{
    ast::{
        expression::Block as AstBlock,
        expression::Expression as AstExpression,
        item::Function as AstFunction,
        item::{ItemKind, Parameter},
        statement::LetStatement,
        statement::Statement as AstStatement,
    },
    hir,
    item_table::ItemTable,
};

use super::{
    scope::Scope,
    types::{TypeError, TypeTable},
    Expression, Function, HirData, Statement,
};

use thiserror::Error;

pub fn translate(item_table: ItemTable) -> Result<HirData, TranslationError> {
    let mut type_table = TypeTable::gather(&item_table)?;
    let mut functions = Vec::new();
    for (_, item) in item_table.into_iter() {
        if let ItemKind::Function(func) = item.kind {
            functions.push(translate_func(func, &mut type_table)?)
        }
    }
    Ok(HirData {
        type_table,
        functions,
    })
}

fn translate_func(
    func: AstFunction,
    type_table: &mut TypeTable,
) -> Result<Function, TranslationError> {
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
            .map(|Parameter { name, type_ }| Ok::<_, TypeError>((name, type_table.get(type_)?)))
            .collect::<Result<_, _>>()?,
        return_type: match return_type {
            Some(type_) => Some(type_table.get(type_)?),
            None => None,
        },
        body: translate_block(type_table, Scope::new(), body)?,
    })
}

fn translate_block(
    type_table: &mut TypeTable,
    scope: Scope,
    block: AstBlock,
) -> Result<hir::Block, TranslationError> {
    let scope = scope.child();
    let mut result = Vec::new();
    for stmt in block.statements {
        let stmt = translate_stmt(type_table, scope.clone(), stmt)?;
        result.push(stmt);
    }
    if let Some(expr) = block.expression {
        let expr = translate_expr(type_table, scope, *expr)?;
        result.push(hir::Statement::Return(expr))
    }
    Ok(hir::Block(result))
}

fn translate_stmt(
    type_table: &mut TypeTable,
    scope: Scope,
    stmt: AstStatement,
) -> Result<hir::Statement, TranslationError> {
    match stmt {
        AstStatement::ExprStmt(expr) => {
            translate_expr(type_table, scope, expr).map(Statement::ExprStmt)
        }
        AstStatement::LetStmt(LetStatement { name, type_, value }) => {
            let Some(type_) = type_ else { return Err(TranslationError::TypeInference)};
            let type_ = type_table.get(type_)?;
            let value = value
                .map(|v| *v)
                .and_then(|expr| translate_expr(type_table, scope, expr).ok())
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

fn translate_expr(
    type_table: &mut TypeTable,
    scope: Scope,
    expr: AstExpression,
) -> Result<hir::Expression, TranslationError> {
    Ok(match expr {
        AstExpression::Block(block) => {
            hir::Expression::Block(translate_block(type_table, scope, block)?)
        }
        AstExpression::If {
            condition,
            body,
            else_body,
        } => todo!(),
        AstExpression::While { condition, body } => todo!(),
        AstExpression::For { var, expr, body } => todo!(),
        AstExpression::Unary { op, value } => todo!(),
        AstExpression::Binary { op, left, right } => todo!(),
        AstExpression::FnCall {
            name,
            params: ast_params,
        } => {
            let mut params = Vec::with_capacity(ast_params.capacity());
            for param in ast_params {
                params.push(translate_expr(type_table, scope.clone(), param)?);
            }
            Expression::FnCall(name, params)
        }
        AstExpression::Var(name) => todo!(),
        AstExpression::Literal(lit) => Expression::Literal(lit),
    })
}

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("type inference is not implemented yet, so type annotation is required for every variable binding")]
    TypeInference,
    #[error(transparent)]
    TypeError(#[from] TypeError),
}
