use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use crate::ast::{expressions::Identifier, item::Item};

/// Symbol table stores all items known to compiler.
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
        scope.push(Identifier(item.name().to_string()));
        self.declared.insert(scope, item);
    }
}

/// Path to Item.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(Vec<Identifier>);

impl Path {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, ident: Identifier) {
        self.0.push(ident);
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
    use crate::ast::expressions::Identifier;
    use super::Path;

    #[test]
    fn display() {
        let mut path = Path::new();
        path.push(Identifier(String::from("crate_name")));
        path.push(Identifier(String::from("module1_name")));
        path.push(Identifier(String::from("module2_name")));
        assert_eq!(String::from("crate_name::module1_name::module2_name"), path.to_string());
    }
}
