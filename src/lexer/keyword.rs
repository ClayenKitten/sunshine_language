use strum::{Display, EnumString};

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
    Break,
    Return,
    Pub,
    Struct,
    Mod,
    True,
    False,
}
