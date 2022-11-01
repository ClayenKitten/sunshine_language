use crate::ast::expressions::Identifier;

/// struct NAME { FIELD: TYPE }
#[derive(Debug, PartialEq, Eq)]
pub struct Struct {
    name: Identifier,
    fields: Vec<StructField>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructField {
    name: Identifier,
    type_: Identifier,
}