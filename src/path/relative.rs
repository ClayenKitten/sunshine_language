use std::fmt::Display;

use crate::Identifier;

/// A relative path that is interpreted differently depending on context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelativePath {
    start: RelativePathStart,
    other: Vec<Identifier>
}

/// First segment of the relative path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelativePathStart {
    Crate,
    Super(usize),
    Identifier(Identifier),
}

impl Display for RelativePathStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelativePathStart::Crate => write!(f, "crate"),
            RelativePathStart::Super(n) => {
                let mut s = "super::".repeat(*n);
                s.pop();
                s.pop();
                write!(f, "{s}")
            },
            RelativePathStart::Identifier(s) => write!(f, "{s}"),
        }
    }
}

impl RelativePath {
    pub fn new(first: RelativePathStart) -> Self {
        Self {
            start: first,
            other: Vec::new(),
        }
    }

    pub fn push(&mut self, ident: Identifier) {
        self.other.push(ident);
    }

    pub fn pop(&mut self) -> Option<Identifier> {
        self.other.pop()
    }
}

impl Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.start)?;
        for entry in self.other.iter() {
            write!(f, "::{}", entry)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{path::relative::{RelativePath, RelativePathStart}, Identifier};

    #[test]
    fn display_start_with_crate() {
        let mut path = RelativePath::new(RelativePathStart::Crate);
        path.push(Identifier(String::from("module1_name")));
        path.push(Identifier(String::from("module2_name")));
        assert_eq!(
            String::from("crate::module1_name::module2_name"),
            path.to_string()
        );
    }

    #[test]
    fn display_start_with_super() {
        let mut path = RelativePath::new(RelativePathStart::Super(3));
        path.push(Identifier(String::from("module1_name")));
        path.push(Identifier(String::from("module2_name")));
        assert_eq!(
            String::from("super::super::super::module1_name::module2_name"),
            path.to_string()
        );
    }
}
