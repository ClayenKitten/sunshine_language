use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{hir::types::TypeId, Identifier};

/// The scope is a portion of code that defines where local variable names are accessible.
///
/// # Lexical scoping
///
/// Sunshine language follows lexical scoping rules for variables.
/// As such, only variables defined in the same or parent scope may be
/// accessed.
///
/// ```
/// fn example() {
///     let a: u8 = 5;
///     // Variable may be accessed in the same scope it was defined.
///     if a < 5 {
///         // It may be accessed in the child scope as well.
///         let b = a + 5;
///     }
///     // However, variable cannot be accessed in the outer scope.
///     # let b = 0;
///     let c = a + b;
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    inner: Rc<RefCell<ScopeInner>>,
    latest_id: Rc<Cell<u32>>,
    loop_context: bool,
}

impl Scope {
    /// Creates a new top-level scope.
    pub fn new() -> Self {
        Scope {
            inner: Rc::new(RefCell::new(ScopeInner {
                parent: None,
                mapping: HashMap::new(),
                types: HashMap::new(),
            })),
            latest_id: Rc::new(Cell::new(0)),
            loop_context: false,
        }
    }

    /// Creates a child scope.
    pub fn child(&self) -> Self {
        Scope {
            inner: Rc::new(RefCell::new(ScopeInner {
                parent: Some(self.clone()),
                mapping: HashMap::new(),
                types: HashMap::new(),
            })),
            latest_id: Rc::clone(&self.latest_id),
            loop_context: self.loop_context,
        }
    }

    /// Creates a child scope that is inside loop.
    pub fn child_loop(&self) -> Self {
        Scope {
            inner: Rc::new(RefCell::new(ScopeInner {
                parent: Some(self.clone()),
                mapping: HashMap::new(),
                types: HashMap::new(),
            })),
            latest_id: Rc::clone(&self.latest_id),
            loop_context: true,
        }
    }

    /// Inserts variable in the scope.
    pub fn insert(&mut self, var: Identifier, type_id: TypeId) -> VarId {
        let mut scope = self.inner.borrow_mut();

        let var_id = VarId(self.latest_id.get());
        scope.mapping.insert(var, var_id);
        scope.types.insert(var_id, type_id);
        self.latest_id.set(var_id.0 + 1);
        var_id
    }

    /// Looks variable up in the scope or one of its parents.
    pub fn lookup(&self, var: &Identifier) -> Option<(VarId, TypeId)> {
        let scope = self.inner.borrow();

        let var_id = scope.mapping.get(var).copied();
        match var_id {
            Some(var_id) => {
                let type_id = scope
                    .types
                    .get(&var_id)
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
        self.inner.borrow().parent.clone()
    }

    /// Checks if current scope is in loop context.
    ///
    /// That, for example, defines if `break` may be used.
    pub fn is_loop(&self) -> bool {
        self.loop_context
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
