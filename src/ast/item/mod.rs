mod module;
mod function;
mod r#struct;

pub use self::{module::Module, r#struct::{Struct, Field}, function::{Function, Parameter}};

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
