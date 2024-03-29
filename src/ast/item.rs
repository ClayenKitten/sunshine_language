use crate::{ast::expression::Block, util::Span, Identifier};

/// An Item is a static component of the package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
    pub visibility: Visibility,
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
            ItemKind::Module(Module::Inline(ident)) => ident,
            ItemKind::Module(Module::Loadable(ident)) => ident,
            ItemKind::Struct(s) => &s.name,
            ItemKind::Function(f) => &f.name,
        }
    }

    pub fn new(item: impl Into<ItemKind>, span: Span, visibility: Visibility) -> Self {
        Self {
            kind: item.into(),
            span,
            visibility,
        }
    }
}

/// Module is a container for zero or more [items](Item).
///
/// Module may be either inline or loadable from separate file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Module {
    Inline(Identifier),
    Loadable(Identifier),
}

impl From<Module> for ItemKind {
    fn from(val: Module) -> Self {
        ItemKind::Module(val)
    }
}

/// A type that is composed of other types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<Field>,
}

impl From<Struct> for ItemKind {
    fn from(val: Struct) -> Self {
        ItemKind::Struct(val)
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

impl From<Function> for ItemKind {
    fn from(val: Function) -> Self {
        ItemKind::Function(val)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Visibility {
    Public,
    #[default]
    Private,
}

#[cfg(test)]
mod test {
    #[test]
    fn visibility_ordering() {
        use super::Visibility::*;
        let expected = vec![Public, Public, Private, Private];
        let mut init = vec![Private, Public, Private, Public];
        init.sort();
        assert_eq!(expected, init);
    }
}
