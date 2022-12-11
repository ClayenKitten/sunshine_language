use crate::{lexer::Lexer, ast::{expressions::Identifier}, parser::ParserError};

use super::Item;

/// Module is a scoped list of items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: Identifier,
}

impl Module {
    /// Parse module. Keyword `mod` is expected to be consumed beforehand.
    #[deprecated = "use Parser::parse_module"]
    pub fn parse(lexer: &mut Lexer) -> Result<Module, ParserError> {
        let name = lexer.expect_identifier()?;
        lexer.expect_punctuation("{")?;
        let mut content = Vec::new();
        loop {
            if lexer.consume_punctuation("}")? {
                break;
            }
            content.push(Item::parse(lexer)?);
        }
        Ok(Module { name })
    }
}
