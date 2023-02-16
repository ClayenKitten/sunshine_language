use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{Identifier, item_table::path::ItemPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope(Rc<RefCell<ScopeInner>>);

impl Scope {
    /// Creates a new top-level scope.
    pub fn new() -> Self {
        Scope(Rc::new(RefCell::new(ScopeInner {
            parent: None,
            vars: HashMap::new(),
        })))
    }

    /// Creates a child scope.
    pub fn child(&self) -> Self {
        Scope(Rc::new(RefCell::new(ScopeInner {
            parent: Some(self.clone()),
            vars: HashMap::new(),
        })))
    }

    /// Inserts variable in the scope.
    pub fn insert(&mut self, var: Identifier, type_: ItemPath) {
        self.0.borrow_mut().vars.insert(var, type_);
    }

    /// Looks variable up in the scope or one of its parents.
    pub fn lookup(&self, var: &Identifier) -> Option<ItemPath> {
        self.0
            .borrow()
            .vars
            .get(var)
            .cloned()
            .or_else(|| self.parent()?.lookup(var))
    }

    /// Gets the parent scope if there is one.
    pub fn parent(&self) -> Option<Scope> {
        self.0.borrow().parent.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeInner {
    parent: Option<Scope>,
    vars: HashMap<Identifier, ItemPath>,
}
