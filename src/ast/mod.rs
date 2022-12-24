pub mod expression;
pub mod identifier;
pub mod item;
pub mod statement;

pub use identifier::*;

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
