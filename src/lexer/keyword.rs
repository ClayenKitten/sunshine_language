use strum::{EnumString, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    Let,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Struct,
    True,
    False,
}
