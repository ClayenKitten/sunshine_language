use std::{
    collections::{hash_map, HashMap},
    fmt::Display,
};

use crate::ast::item::Item;

/// Symbol table stores all items known to compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable {
    pub declared: HashMap<path::ItemPath, Item>,
    duplicated: Vec<(path::ItemPath, Item)>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            declared: HashMap::new(),
            duplicated: Vec::new(),
        }
    }

    /// Merge two symbol tables.
    pub fn extend(&mut self, other: SymbolTable) {
        self.duplicated.extend(other.duplicated.into_iter());

        self.declared.reserve(other.declared.len());
        for (path, item) in other.declared {
            self.try_insert(path, item);
        }
    }

    /// Add new entry to symbol table.
    ///
    /// `scope` is path to `item`'s parent.
    pub fn declare(&mut self, mut scope: path::ItemPath, item: Item) {
        scope.push(item.name().clone());
        self.try_insert(scope, item);
    }

    pub fn declare_anonymous(&mut self, scope: path::ItemPath, item: Item) {
        self.try_insert(scope, item);
    }

    /// Try to insert provided [Item] to `declared`. If it already exists, push it to `duplicated`
    /// instead.
    fn try_insert(&mut self, path: path::ItemPath, item: Item) {
        if self.declared.contains_key(&path) {
            self.duplicated.push((path, item));
        } else {
            self.declared.insert(path, item);
        }
    }

    pub fn iter(&self) -> hash_map::Iter<path::ItemPath, Item> {
        self.declared.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<path::ItemPath, Item> {
        self.declared.iter_mut()
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (path, item) in self.declared.iter() {
            writeln!(f, "{}\n{:#?}", path, item)?;
        }
        Ok(())
    }
}

pub mod path {
    use itertools::Itertools;
    use std::fmt::Display;
    use std::iter::once;
    use std::path::PathBuf;
    use std::slice;
    use std::str::FromStr;
    use thiserror::Error;

    use crate::ast::{Identifier, IdentifierParseError};

    /// Path to Item.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ItemPath {
        pub(crate) krate: Identifier,
        pub(crate) other: Vec<Identifier>,
    }

    impl ItemPath {
        pub fn new(krate: impl Into<Identifier>) -> Self {
            Self {
                krate: krate.into(),
                other: Vec::new(),
            }
        }

        pub fn push(&mut self, ident: Identifier) {
            self.other.push(ident);
        }

        pub fn pop(&mut self) -> Option<Identifier> {
            self.other.pop()
        }

        pub fn last(&self) -> &Identifier {
            self.other.last()
                .unwrap_or(&self.krate)
        }

        pub fn iter(&self) -> slice::Iter<Identifier> {
            self.other.iter()
        }

        /// Map that [Path] to system's [PathBuf] relative to the main source file.
        pub fn into_path_buf(self) -> PathBuf {
            self.other.into_iter().map(|ident| ident.0).collect()
        }
    }

    impl Display for ItemPath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            #[allow(unstable_name_collisions)]
            once(&self.krate)
                .chain(self.other.iter())
                .map(|ident| ident.0.as_str())
                .intersperse("::")
                .try_for_each(|s| write!(f, "{}", s))
        }
    }

    impl FromStr for ItemPath {
        type Err = ItemPathParsingError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut entries = s.split("::");
            let krate = entries
                .next()
                .ok_or(ItemPathParsingError::ExpectedIdentifier)
                .and_then(|s| {
                    if s.is_empty() {
                        Err(ItemPathParsingError::ExpectedIdentifier)
                    } else {
                        Ok(s)
                    }
                })
                .and_then(|s| Identifier::from_str(s).map_err(Into::into))?;
            let other = entries
                .map(|s| {
                    Identifier::from_str(s).map_err(|e| {
                        if e == IdentifierParseError::Empty {
                            ItemPathParsingError::ExpectedIdentifier
                        } else {
                            ItemPathParsingError::InvalidIdentifier(e)
                        }
                    })
                })
                .collect::<Result<_, _>>()?;
            Ok(ItemPath { krate, other })
        }
    }

    #[derive(Debug, PartialEq, Eq, Error)]
    pub enum ItemPathParsingError {
        #[error("expected identifier")]
        ExpectedIdentifier,
        #[error("invalid identifier, {0}")]
        InvalidIdentifier(#[from] IdentifierParseError),
    }

    #[cfg(test)]
    mod test {
        use std::str::FromStr;

        use crate::{ast::Identifier, symbol_table::path::ItemPath};

        #[test]
        fn display() {
            let mut path = ItemPath::new(Identifier(String::from("crate")));
            path.push(Identifier(String::from("module1_name")));
            path.push(Identifier(String::from("module2_name")));
            assert_eq!(
                String::from("crate::module1_name::module2_name"),
                path.to_string()
            );
        }

        #[test]
        fn from_str() {
            let mut path = ItemPath::new(Identifier(String::from("crate")));
            path.push(Identifier(String::from("module1_name")));
            path.push(Identifier(String::from("module2_name")));
            assert_eq!(
                path,
                ItemPath::from_str("crate::module1_name::module2_name").unwrap()
            )
        }
    }
}
