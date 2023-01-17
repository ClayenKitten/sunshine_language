//! Block is an expression with attached SymbolTable for its contents.

use std::{collections::HashMap, sync::Arc};

use crate::item_table::path::ItemPath;

use super::{expression::Expression, statement::Statement, Identifier};

/// Block is an expression that consists of a number of statements and an optional final expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
    pub scope: Scope,
}

/// Scope of variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope(Arc<ScopeInner>);

impl Scope {
    /// Creates new top-level scope.
    pub fn new() -> Scope {
        Scope(Arc::new(ScopeInner {
            parent: None,
            variables: HashMap::new(),
        }))
    }

    /// Creates new subscope.
    pub fn new_subscope(parent: &Scope) -> Scope {
        Scope(Arc::new(ScopeInner {
            parent: Some(parent.clone()),
            variables: HashMap::new(),
        }))
    }

    /// Adds new variable declaration.
    ///
    /// If `ident` is already used in current scope, it is replaced. `Ident`s declared in outer scope
    /// aren't replaced, but are shadowed by [`find`](Scope::find) method implementation.
    pub fn declare(&self, ident: Identifier, type_: ItemPath) {
        self.0.variables.insert(ident, type_);
    }

    /// Finds declaration in scope or one of its parents.
    pub fn find(&self, ident: &Identifier) -> Option<ItemPath> {
        if let Some(type_) = self.0.variables.get(ident) {
            Some(type_.clone())
        } else if let Some(parent) = self.0.parent {
            parent.find(ident)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeInner {
    parent: Option<Scope>,
    variables: HashMap<Identifier, ItemPath>,
}
