//! Symbol table of [items](Item).
//!
//! Items have special scope and visibility rules as opposed to variable bindings.
//! As such, they are stored in special data structure.

use std::{
    collections::{
        hash_map::{self, Entry},
        HashMap,
    },
    fmt::Display,
};

use crate::ast::item::Item;

use self::path::ItemPath;

/// Table of all known items.
///
/// See the [module documentation] for details.
///
/// [module documentation]: crate::item_table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTable {
    pub declared: HashMap<ItemPath, Item>,
    duplicated: Vec<(ItemPath, Item)>,
}

impl ItemTable {
    pub fn new() -> Self {
        ItemTable {
            declared: HashMap::new(),
            duplicated: Vec::new(),
        }
    }

    /// Merge two item tables.
    pub fn extend(&mut self, other: ItemTable) {
        self.duplicated.extend(other.duplicated.into_iter());

        self.declared.reserve(other.declared.len());
        for (path, item) in other.declared {
            self.try_insert(path, item);
        }
    }

    /// Add new entry to item table.
    ///
    /// `scope` is path to `item`'s parent.
    pub fn declare(&mut self, mut scope: ItemPath, item: Item) {
        scope.push(item.name().clone());
        self.try_insert(scope, item);
    }

    pub fn declare_anonymous(&mut self, scope: ItemPath, item: Item) {
        self.try_insert(scope, item);
    }

    /// Try to insert provided [Item] to `declared`. If it already exists, push it to `duplicated`
    /// instead.
    fn try_insert(&mut self, path: ItemPath, item: Item) {
        match self.declared.entry(path) {
            Entry::Vacant(entry) => {
                entry.insert(item);
            }
            Entry::Occupied(entry) => self.duplicated.push((entry.key().clone(), item)),
        }
    }

    pub fn items(&self) -> hash_map::Values<ItemPath, Item> {
        self.declared.values()
    }

    pub fn iter(&self) -> hash_map::Iter<ItemPath, Item> {
        self.declared.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<ItemPath, Item> {
        self.declared.iter_mut()
    }
}

impl Default for ItemTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ItemTable {
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
            self.other.last().unwrap_or(&self.krate)
        }

        pub fn iter(&self) -> slice::Iter<Identifier> {
            self.other.iter()
        }

        /// Maps [ItemPath] into relative [PathBuf].
        ///
        /// # Example
        ///
        /// ```rust
        /// # use std::path::PathBuf;
        /// # use compiler::{ast::Identifier, item_table::path::ItemPath};
        /// let mut path = ItemPath::new(Identifier(String::from("example")));
        /// path.push(Identifier(String::from("mod1")));
        /// path.push(Identifier(String::from("mod2")));
        ///
        /// assert_eq!(
        ///     path.into_path_buf(),
        ///     PathBuf::from("mod1/mod2.sun"),
        /// );
        /// ```
        pub fn into_path_buf(self) -> PathBuf {
            let mut path: PathBuf = self.other.into_iter().map(|ident| ident.0).collect();
            path.set_extension("sun");
            path
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

        use crate::{ast::Identifier, item_table::path::ItemPath};

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
