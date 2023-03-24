use std::{borrow::Borrow, collections::HashMap, str::FromStr};

use crate::{
    ast::item::{Field, ItemKind, Struct},
    item_table::ItemTable,
    util::MonotonicVec,
    Identifier,
};

use thiserror::Error;

#[derive(Debug)]
pub struct TypeTable {
    latest_compound: u32,
    mapping: HashMap<Identifier, TypeId>,
    fields: MonotonicVec<HashMap<Identifier, TypeId>>,
}

impl TypeTable {
    /// Creates new [TypeTable] by gathering type declarations from provided [ItemTable].
    pub fn gather(item_table: &ItemTable) -> TypeResult<TypeTable> {
        let mut type_table = TypeTable {
            latest_compound: 0,
            fields: MonotonicVec::new(),
            mapping: HashMap::new(),
        };
        for item in item_table.items() {
            if let ItemKind::Struct(Struct { ref name, .. }) = item.kind {
                type_table.define(name.clone());
            }
        }
        for item in item_table.items() {
            if let ItemKind::Struct(Struct {
                ref name,
                ref fields,
            }) = item.kind
            {
                let strukt = type_table.get(name)?;
                for Field { name, type_ } in fields {
                    type_table.add_field(strukt, name.clone(), type_.clone())?;
                }
            }
        }
        Ok(type_table)
    }

    pub fn get(&self, id: impl Borrow<Identifier>) -> TypeResult<TypeId> {
        self.mapping
            .get(id.borrow())
            .copied()
            .or_else(|| {
                PrimitiveType::from_str(id.borrow().as_str())
                    .ok()
                    .map(|t| TypeId::Primitive(t))
            })
            .ok_or_else(|| TypeError::NotFound(id.borrow().clone()))
    }

    /// Defines the compound type without its internals.
    fn define(&mut self, name: Identifier) -> TypeId {
        let id = TypeId::Compound(self.latest_compound);
        self.mapping.insert(name, id);
        self.fields.push(HashMap::default());
        self.latest_compound += 1;
        id
    }

    /// Adds field of the struct to the table.
    fn add_field(&mut self, strukt: TypeId, name: Identifier, type_: Identifier) -> TypeResult<()> {
        let type_ = self.get(type_)?;
        if let TypeId::Compound(index) = strukt {
            self.fields[index as usize].insert(name, type_);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeId {
    Primitive(PrimitiveType),
    Compound(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
}

impl FromStr for PrimitiveType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PrimitiveType::*;
        Ok(match s {
            "bool" => Bool,
            "u8" => U8,
            "u16" => U16,
            "u32" => U32,
            "u64" => U64,
            "usize" => Usize,
            "i8" => I8,
            "i16" => I16,
            "i32" => I32,
            "i64" => I64,
            "isize" => Isize,
            _ => return Err(()),
        })
    }
}

pub type TypeResult<T> = Result<T, TypeError>;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("type `{0}` is not found")]
    NotFound(Identifier),
    #[error("type `{0}` is already defined")]
    AlreadyDefined(Identifier),
}
