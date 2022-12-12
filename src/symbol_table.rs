use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use crate::ast::{Identifier, item::Item};

/// Symbol table stores all items known to compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable {
    declared: HashMap<Path, Item>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable { declared: HashMap::new() }
    }

    /// Add new entry to symbol table.
    /// 
    /// `scope` is path to `item`'s parent.
    pub fn declare(&mut self, mut scope: Path, item: Item) {
        scope.push(item.name().clone());
        self.declared.insert(scope, item);
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
pub struct Path(Vec<Identifier>);

impl Path {
    pub fn new() -> Self {
        Self(vec![Identifier(String::from("crate"))])
    }

    pub fn push(&mut self, ident: Identifier) {
        self.0.push(ident);
    }

    pub fn pop(&mut self) -> Identifier {
        if self.0.len() > 1 {
            return self.0.pop().unwrap();
        }
        self.0.last().unwrap().clone()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unstable_name_collisions)]
        self.0.iter()
            .map(|ident| ident.0.as_str())
            .intersperse("::")
            .try_for_each(|s| write!(f, "{}", s))
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Identifier;
    use super::Path;

    #[test]
    fn display() {
        let mut path = Path::new();
        path.push(Identifier(String::from("module1_name")));
        path.push(Identifier(String::from("module2_name")));
        assert_eq!(String::from("crate::module1_name::module2_name"), path.to_string());
    }
}
