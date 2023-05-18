//! High-level intermediate representation.
//!
//! AST to HIR translation includes type checking and desugaring.

mod builder;
pub mod scope;
pub mod types;

pub use builder::{HirBuilder, TranslationError};

use crate::ast::expression::Literal;

use self::{
    scope::VarId,
    types::{TypeId, TypeTable},
};

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
    pub params: Vec<(VarId, TypeId)>,
    pub return_type: Option<TypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Expression {
    type_: Option<TypeId>,
    kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpressionKind {
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
        assignee: VarId,
        value: Expression,
    },
    Return(Expression),
    Break,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Block(Vec<Statement>, Option<Box<Expression>>);

impl Block {
    pub fn type_id(&self) -> Option<TypeId> {
        self.1.as_ref().and_then(|expr| expr.type_)
    }
}
