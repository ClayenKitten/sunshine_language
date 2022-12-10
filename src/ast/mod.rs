use self::{expressions::*, item::Module};

pub mod expressions;
pub mod item;
pub mod statement;

#[derive(Debug)]
pub struct Ast(pub Module);
