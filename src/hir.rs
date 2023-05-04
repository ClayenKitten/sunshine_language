//! High-level intermediate representation.
//!
//! AST to HIR translation includes type checking and desugaring.

mod builder;
pub mod scope;
pub mod types;

pub use builder::{HirBuilder, TranslationError};

use std::collections::HashMap;

use crate::{ast::expression::Literal, Identifier};

use self::{types::{TypeId, TypeTable}, scope::VarId};

#[derive(Debug, Default)]
pub struct Hir {
    type_table: TypeTable,
    functions: Vec<Function>,
}

impl Hir {
    pub fn get_function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.get(id.0 as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionId(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub signature: FunctionSignature,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub params: HashMap<Identifier, TypeId>,
    pub return_type: Option<TypeId>,
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
    FnCall(FunctionId, Vec<Expression>),
    Var(VarId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Statement {
    ExprStmt(Expression),
    LetStmt {
        var: VarId,
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
