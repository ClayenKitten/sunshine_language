use crate::lexer::{TokenStream, Token, punctuation::Punctuation};

use super::{item::Item, Expression, expressions::LetStatement, ParserError, Delimiter};

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Item(Item),
    ExpressionStatement(Expression),
    LetStatement(LetStatement),
}

impl Statement {
    /// Parse statements until closing delimiter met.
    pub fn parse_block(lexer: &mut TokenStream, delimiter: Delimiter) -> Result<Vec<Statement>, ParserError> {
        let mut buffer = Vec::new();
        loop {
            let token = lexer.next()?;
            if let Token::Punctuation(Punctuation(punc)) = token {
                if delimiter.is_matching_closing_delimiter(punc) {
                    break;
                }
            }

            todo!();
        }
        Ok(buffer)
    }
}
