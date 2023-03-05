//! High-level intermediate representation.
//!
//! AST to HIR translation includes type checking and desugaring.

pub mod scope;
mod translation;
pub mod types;

use std::collections::HashMap;

use crate::{ast::expression::Literal, item_table::ItemTable, Identifier};

use self::{
    translation::TranslationError,
    types::{TypeId, TypeTable},
};

#[derive(Debug)]
pub struct HirData {
    type_table: TypeTable,
    functions: Vec<Function>,
}

impl HirData {
    pub fn translate(item_table: ItemTable) -> Result<Self, TranslationError> {
        translation::translate(item_table)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarId(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub params: HashMap<Identifier, TypeId>,
    pub return_type: Option<TypeId>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expression {
    Block(Block),
    If {
        condition: Box<Expression>,
        body: Block,
        else_body: Option<Block>,
    },
    Loop(Block),
    Literal(Literal),
    FnCall(Identifier, Vec<Expression>),
    Var(VarId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Statement {
    ExprStmt(Expression),
    LetStmt {
        name: Identifier,
        type_: TypeId,
        value: Option<Box<Expression>>,
    },
    Assignment {
        assignee: Identifier,
        value: Expression,
    },
    Return(Expression),
    Break,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Block(Vec<Statement>);
