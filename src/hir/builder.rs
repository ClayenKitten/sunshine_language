mod body;

use std::collections::HashMap;

use crate::{
    ast::{
        expression::Block as AstBlock,
        item::Function as AstFunction,
        item::{Field, ItemKind, Parameter},
    },
    item_table::ItemTable,
    path::AbsolutePath,
    Identifier,
};

use self::body::BodyBuilder;

use super::{
    types::{TypeError, TypeId, TypeTable},
    Block, Function, FunctionId, Hir,
};

use thiserror::Error;

#[derive(Debug, Default)]
pub struct HirBuilder {
    type_table: TypeTable,
    errors: Vec<TranslationError>,

    mapping: HashMap<AbsolutePath, FunctionId>,
    signatures: Vec<(Vec<TypeId>, Option<TypeId>)>,
    bodies: Vec<Block>,
}

impl HirBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Result<Hir, Vec<TranslationError>> {
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        let HirBuilder {
            type_table,
            signatures,
            bodies,
            ..
        } = self;
        debug_assert_eq!(signatures.len(), bodies.len());

        let functions = signatures
            .into_iter()
            .zip(bodies)
            .map(|((params, return_type), body)| Function {
                params,
                return_type,
                body,
            })
            .collect();

        Ok(Hir {
            type_table,
            functions,
        })
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
                    let id = FunctionId(self.mapping.len() as u32);
                    self.mapping.insert(path.clone(), id);
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

        let mut partial_functions = Vec::with_capacity(functions.len());
        for (path, function) in functions {
            match self.partially_translate_function(path, function) {
                Ok(partial) => {
                    let params = partial.params.iter().map(|(_, type_id)| *type_id).collect();
                    let return_type = partial.return_type;
                    self.signatures.push((params, return_type));
                    partial_functions.push(partial);
                }
                Err(err) => self.errors.push(err),
            }
        }

        for partial in partial_functions {
            match BodyBuilder::translate(self, partial) {
                Ok(body) => self.bodies.push(body),
                Err(error) => self.errors.push(error),
            }
        }
    }

    fn partially_translate_function(
        &self,
        mut path: AbsolutePath,
        func: AstFunction,
    ) -> Result<PartiallyParsedFunction, TranslationError> {
        let mut partial_func = PartiallyParsedFunction {
            module: {
                path.pop();
                path
            },
            params: Vec::with_capacity(func.params.len()),
            return_type: None,
            body: func.body,
        };

        for Parameter { name, type_ } in func.params {
            let type_id = self.type_table.get(type_)?;
            partial_func.params.push((name, type_id))
        }
        partial_func.return_type = func
            .return_type
            .map(|type_| self.type_table.get(type_))
            .transpose()?;

        Ok(partial_func)
    }

    fn query_function_info(
        &self,
        path: &AbsolutePath,
    ) -> Option<(FunctionId, &[TypeId], Option<TypeId>)> {
        let id = self.mapping.get(path).copied()?;
        let signature = &self.signatures[id.0 as usize];
        Some((id, signature.0.as_slice(), signature.1))
    }
}

struct PartiallyParsedFunction {
    pub module: AbsolutePath,
    pub params: Vec<(Identifier, TypeId)>,
    pub return_type: Option<TypeId>,
    pub body: AstBlock,
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
    #[error("`if` and `else` have incompatible types. Expected {body:?}, found {else_body:?}.")]
    IfBranchTypeMismatch {
        body: Option<TypeId>,
        else_body: Option<TypeId>,
    },
    #[error("incorrect number of arguments provided for function. Expected {expected:?}, received {received:?}.")]
    ArgumentCountMismatch { expected: usize, received: usize },
    #[error("variable `{0}` is not declared")]
    VariableNotDeclared(Identifier),
    #[error("function {0} is not found")]
    FunctionNotFound(AbsolutePath),
    #[error("break may not be used outside of the loop")]
    InvalidBreak,
    #[error(transparent)]
    TypeError(#[from] TypeError),
}
