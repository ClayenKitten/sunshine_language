use crate::ast::expressions::Identifier;

/// A type that is composed of other types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<Field>,
}

/// Field 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Identifier,
    pub type_: Identifier,
}
