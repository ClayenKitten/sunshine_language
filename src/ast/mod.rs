use std::fmt::Display;

pub mod expression;
pub mod item;
pub mod statement;

/// Identifier is name of type, variable or function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Visibility {
    Public,
    #[default]
    Private,
}

#[cfg(test)]
mod test {
    #[test]
    fn visibility_ordering() {
        use super::Visibility::*;
        let expected = vec![Public, Public, Private, Private];
        let mut init = vec![Private, Public, Private, Public];
        init.sort();
        assert_eq!(expected, init);
    }
}
