use super::{expressions::Identifier, statement::Block};

/// An Item is a static component of the package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Module(Module),
    Struct(Struct),
    Function(Function),
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Item::Module(m) => &m.name.0,
            Item::Struct(s) => &s.name.0,
            Item::Function(f) => &f.name.0,
        }
    }
}

/// Module is a scoped list of items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: Identifier,
}

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

/// A parameter represents a value that the function expects you to pass when you call it.
///
/// `NAME: TYPE`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Identifier,
    pub type_: Identifier,
}
