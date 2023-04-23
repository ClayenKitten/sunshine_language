use itertools::Itertools;
use std::fmt::Display;
use std::iter::once;
use std::path::PathBuf;
use std::slice;
use std::str::FromStr;

use crate::identifier::{Identifier, IdentifierParseError};

use super::PathParsingError;

/// A fully qualified path that indicates specific item.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePath {
    pub(crate) krate: Identifier,
    pub(crate) other: Vec<Identifier>,
}

impl AbsolutePath {
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

    /// Maps [AbsolutePath] into relative [PathBuf].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::path::PathBuf;
    /// # use compiler::{Identifier, path::AbsolutePath};
    /// let mut path = AbsolutePath::new(Identifier(String::from("example")));
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

impl Display for AbsolutePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unstable_name_collisions)]
        once(&self.krate)
            .chain(self.other.iter())
            .map(|ident| ident.0.as_str())
            .intersperse("::")
            .try_for_each(|s| write!(f, "{s}"))
    }
}

impl FromStr for AbsolutePath {
    type Err = PathParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut entries = s.split("::");
        let krate = entries
            .next()
            .ok_or(PathParsingError::ExpectedIdentifier)
            .and_then(|s| {
                if s.is_empty() {
                    Err(PathParsingError::ExpectedIdentifier)
                } else {
                    Ok(s)
                }
            })
            .and_then(|s| Identifier::from_str(s).map_err(Into::into))?;
        let other = entries
            .map(|s| {
                Identifier::from_str(s).map_err(|e| {
                    if e == IdentifierParseError::Empty {
                        PathParsingError::ExpectedIdentifier
                    } else {
                        PathParsingError::InvalidIdentifier(e)
                    }
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(AbsolutePath { krate, other })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{path::AbsolutePath, Identifier};

    #[test]
    fn display() {
        let mut path = AbsolutePath::new(Identifier(String::from("crate")));
        path.push(Identifier(String::from("module1_name")));
        path.push(Identifier(String::from("module2_name")));
        assert_eq!(
            String::from("crate::module1_name::module2_name"),
            path.to_string()
        );
    }

    #[test]
    fn from_str() {
        let mut path = AbsolutePath::new(Identifier(String::from("crate")));
        path.push(Identifier(String::from("module1_name")));
        path.push(Identifier(String::from("module2_name")));
        assert_eq!(
            path,
            AbsolutePath::from_str("crate::module1_name::module2_name").unwrap()
        )
    }
}
