use std::{borrow::Borrow, collections::HashMap, str::FromStr};

use crate::{util::MonotonicVec, Identifier};

use thiserror::Error;

/// Type table is a representation of all types defined in the program.
#[derive(Debug, Default)]
pub struct TypeTable {
    pub(super) latest_compound: u32,
    pub(super) mapping: HashMap<Identifier, TypeId>,
    pub(super) fields: MonotonicVec<HashMap<Identifier, TypeId>>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: impl Borrow<Identifier>) -> TypeResult<TypeId> {
        self.mapping
            .get(id.borrow())
            .copied()
            .or_else(|| {
                PrimitiveType::from_str(id.borrow().as_str())
                    .ok()
                    .map(TypeId::Primitive)
            })
            .ok_or_else(|| TypeError::NotFound(id.borrow().clone()))
    }

    /// Adds user-defined type's name to the table.
    pub(super) fn define_name(&mut self, name: Identifier) -> TypeId {
        let id = TypeId::Compound(self.latest_compound);
        self.mapping.insert(name, id);
        self.fields.push(HashMap::default());
        self.latest_compound += 1;
        id
    }

    /// Adds field to defined struct.
    pub(super) fn add_field(
        &mut self,
        strukt: TypeId,
        name: Identifier,
        type_: Identifier,
    ) -> TypeResult<()> {
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

impl TypeId {
    pub const BOOL: TypeId = TypeId::Primitive(PrimitiveType::Bool);
    pub const I32: TypeId = TypeId::Primitive(PrimitiveType::I32);
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
    F32,
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
            "f32" => F32,
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
