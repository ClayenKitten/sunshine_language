pub mod expression;
pub mod item;
pub mod statement;

/// Identifier is name of type, variable or function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);
