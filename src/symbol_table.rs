use std::{collections::{HashMap, hash_map}, fmt::Display, path::PathBuf, slice, iter::once};

use itertools::Itertools;

use crate::ast::{Identifier, item::Item};

/// Symbol table stores all items known to compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable {
    declared: HashMap<ItemPath, Item>,
    duplicated: Vec<(ItemPath, Item)>,
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
    pub fn declare(&mut self, mut scope: ItemPath, item: Item) {
        scope.push(item.name().clone());
        self.try_insert(scope, item);
    }

    /// Try to insert provided [Item] to `declared`. If it already exists, push it to `duplicated`
    /// instead.
    fn try_insert(&mut self, path: ItemPath, item: Item) {
        if self.declared.contains_key(&path) {
            self.duplicated.push((path, item));
        } else {
            self.declared.insert(path, item);
        }
    }

    pub fn iter(&self) -> hash_map::Iter<ItemPath, Item> {
        self.declared.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<ItemPath, Item> {
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

/// Path to Item.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemPath {
    krate: Identifier,
    other: Vec<Identifier>,
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

    pub fn last(&self) -> Option<&Identifier> {
        self.other.last()
    }

    pub fn iter(&self) -> slice::Iter<Identifier> {
        self.other.iter()
    }

    /// Map that [Path] to system's [PathBuf] relative to the main source file.
    pub fn into_path_buf(self) -> PathBuf {
        self.other.into_iter()
            .map(|ident| ident.0)
            .collect()
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

#[cfg(test)]
mod test {
    use crate::ast::Identifier;
    use super::ItemPath;

    #[test]
    fn display() {
        let mut path = ItemPath::new(Identifier(String::from("crate")));
        path.push(Identifier(String::from("module1_name")));
        path.push(Identifier(String::from("module2_name")));
        assert_eq!(String::from("crate::module1_name::module2_name"), path.to_string());
    }
}
