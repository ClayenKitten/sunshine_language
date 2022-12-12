use super::{Identifier, expression::Block, Visibility};

/// An Item is a static component of the package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    kind: ItemKind,
    visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Module(Module),
    Struct(Struct),
    Function(Function),
}

impl Item {
    pub fn name(&self) -> &Identifier {
        match &self.kind {
            ItemKind::Module(m) => &m.name,
            ItemKind::Struct(s) => &s.name,
            ItemKind::Function(f) => &f.name,
        }
    }

    pub fn new(item: impl Into<ItemKind>, visibility: Visibility) -> Self {
        Self {
            kind: item.into(),
            visibility,
        }
    }
}

/// Module is a scoped list of items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: Identifier,
}

impl Into<ItemKind> for Module {
    fn into(self) -> ItemKind {
        ItemKind::Module(self)
    }
}

/// A type that is composed of other types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<Field>,
}

impl Into<ItemKind> for Struct {
    fn into(self) -> ItemKind {
        ItemKind::Struct(self)
    }
}

/// Field
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Identifier,
    pub type_: Identifier,
}

/// A function is a set of statements to perform a specific task.
///
/// `fn NAME(NAME: TYPE, ...) -> RETURN_TYPE`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub return_type: Option<Identifier>,
    pub body: Block,
}

impl Into<ItemKind> for Function {
    fn into(self) -> ItemKind {
        ItemKind::Function(self)
    }
}

/// A parameter represents a value that the function expects you to pass when you call it.
///
/// `NAME: TYPE`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Identifier,
    pub type_: Identifier,
}
