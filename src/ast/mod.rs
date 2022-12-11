use self::item::Module;

pub mod expression;
pub mod item;
pub mod statement;

#[derive(Debug)]
pub struct Ast(pub Module);
