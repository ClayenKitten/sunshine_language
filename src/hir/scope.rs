use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{hir::types::TypeId, Identifier};

/// The scope is a portion of code that defines where variables are valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope(Rc<RefCell<ScopeInner>>);

impl Scope {
    /// Creates a new top-level scope.
    pub fn new() -> Self {
        Scope(Rc::new(RefCell::new(ScopeInner {
            parent: None,
            mapping: HashMap::new(),
            types: HashMap::new(),
        })))
    }

    /// Creates a child scope.
    pub fn child(&self) -> Self {
        Scope(Rc::new(RefCell::new(ScopeInner {
            parent: Some(self.clone()),
            mapping: HashMap::new(),
            types: HashMap::new(),
        })))
    }

    /// Inserts variable in the scope.
    pub fn insert(&mut self, var: Identifier, type_id: TypeId) -> VarId {
        let mut scope = self.0.borrow_mut();
        
        let var_id = VarId(scope.mapping.len() as u32);
        scope.mapping.insert(var, var_id);
        scope.types.insert(var_id, type_id);
        var_id
    }

    /// Looks variable up in the scope or one of its parents.
    pub fn lookup(&self, var: &Identifier) -> Option<(VarId, TypeId)> {
        let scope = self.0.borrow();

        let var_id = scope.mapping.get(var).copied();
        match var_id {
            Some(var_id) => {
                let type_id = scope.types.get(&var_id)
                    .expect("Type should be defined for any `var_id` defined at the same scope");
                Some((var_id, *type_id))
            }
            None => {
                let Some(ref scope) = scope.parent else { return None; };
                scope.lookup(var)
            }
        }
    }

    /// Gets the parent scope if there is one.
    pub fn parent(&self) -> Option<Scope> {
        self.0.borrow().parent.clone()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeInner {
    parent: Option<Scope>,
    mapping: HashMap<Identifier, VarId>,
    types: HashMap<VarId, TypeId>,
}

/// An id of local variable.
/// 
/// These ids are only unique in the same function they were declared at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(u32);
