mod body;

use std::collections::HashMap;

use crate::{
    ast::{
        item::Function as AstFunction,
        item::{Field, ItemKind},
    },
    item_table::ItemTable,
    path::AbsolutePath,
    Identifier,
};

use self::body::BodyBuilder;

use super::{
    types::{TypeError, TypeId, TypeTable},
    Function, FunctionId, Hir,
};

use thiserror::Error;

#[derive(Debug, Default)]
pub struct HirBuilder {
    type_table: TypeTable,
    function_mapping: HashMap<AbsolutePath, FunctionId>,
    functions: Vec<Function>,
    errors: Vec<TranslationError>,
}

impl HirBuilder {
    pub fn build(self) -> Result<Hir, Vec<TranslationError>> {
        if self.errors.is_empty() {
            Ok(Hir {
                type_table: self.type_table,
                functions: self.functions,
            })
        } else {
            Err(self.errors)
        }
    }

    pub fn populate(&mut self, item_table: ItemTable) {
        let mut strukts: Vec<(TypeId, Vec<Field>)> = Vec::new();
        let mut functions: Vec<(AbsolutePath, AstFunction)> = Vec::new();

        for (path, item) in item_table.into_iter() {
            match item.kind {
                ItemKind::Module(_) => {}
                ItemKind::Struct(strukt) => {
                    let id = self.type_table.define_name(strukt.name.clone());
                    strukts.push((id, strukt.fields));
                }
                ItemKind::Function(function) => {
                    let id = FunctionId(self.function_mapping.len() as u32);
                    self.function_mapping.insert(path.clone(), id);
                    functions.push((path, function));
                }
            }
        }

        for (id, fields) in strukts {
            for Field { name, type_ } in fields {
                let result = self.type_table.add_field(id, name, type_);
                if let Err(err) = result {
                    self.errors.push(err.into());
                }
            }
        }

        for (mut path, function) in functions {
            path.pop();
            match BodyBuilder::translate(self, path, function) {
                Ok(function) => self.functions.push(function),
                Err(error) => self.errors.push(error),
            }
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("type inference is not implemented yet, so type annotation is required for every variable binding")]
    TypeInference,
    #[error("type does not match. Expected {expected:?}, received {received:?}.")]
    TypeMismatch {
        expected: Option<TypeId>,
        received: Option<TypeId>,
    },
    #[error("variable `{0}` is not declared")]
    VariableNotDeclared(Identifier),
    #[error("function {0} is not found")]
    FunctionNotFound(AbsolutePath),
    #[error("break may not be used outside of the loop")]
    InvalidBreak,
    #[error(transparent)]
    TypeError(#[from] TypeError),
}
