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

use crate::path::AbsolutePath;

/// Table of all known items.
///
/// See the [module documentation] for details.
///
/// [module documentation]: crate::item_table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTable {
    pub declared: HashMap<AbsolutePath, Item>,
    duplicated: Vec<(AbsolutePath, Item)>,
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
    pub fn declare(&mut self, mut scope: AbsolutePath, item: Item) {
        scope.push(item.name().clone());
        self.try_insert(scope, item);
    }

    pub fn declare_anonymous(&mut self, scope: AbsolutePath, item: Item) {
        self.try_insert(scope, item);
    }

    /// Try to insert provided [Item] to `declared`. If it already exists, push it to `duplicated`
    /// instead.
    fn try_insert(&mut self, path: AbsolutePath, item: Item) {
        match self.declared.entry(path) {
            Entry::Vacant(entry) => {
                entry.insert(item);
            }
            Entry::Occupied(entry) => self.duplicated.push((entry.key().clone(), item)),
        }
    }

    pub fn items(&self) -> hash_map::Values<AbsolutePath, Item> {
        self.declared.values()
    }

    pub fn iter(&self) -> hash_map::Iter<AbsolutePath, Item> {
        self.declared.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<AbsolutePath, Item> {
        self.declared.iter_mut()
    }
}

impl IntoIterator for ItemTable {
    type Item = (AbsolutePath, Item);
    type IntoIter = hash_map::IntoIter<AbsolutePath, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.declared.into_iter()
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
            writeln!(f, "{path}\n{item:#?}")?;
        }
        Ok(())
    }
}
