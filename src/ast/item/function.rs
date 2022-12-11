use crate::ast::{expressions::Identifier, statement::Block};

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
