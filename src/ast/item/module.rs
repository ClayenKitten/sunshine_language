use crate::ast::expressions::Identifier;

/// Module is a scoped list of items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: Identifier,
}
