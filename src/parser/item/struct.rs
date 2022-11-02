use crate::{parser::{expressions::Identifier, ParserError, UnexpectedTokenError}, lexer::{TokenStream, Token, punctuation::Punctuation}};

/// A type that is composed of other types.
#[derive(Debug, PartialEq, Eq)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<Field>,
}

/// Field 
#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub name: Identifier,
    pub type_: Identifier,
}

impl Struct {
    /// Parse structure from token stream. `struct` keyword is expected to be consumed beforehand.
    pub fn parse(lexer: &mut TokenStream) -> Result<Struct, ParserError> {
        let name = Identifier::parse(lexer)?;
        let mut fields = Vec::new();
        lexer.expect_punctuation(&["{"])?;
        
        loop {
            match Self::parse_field(lexer)? {
                Some(field) => fields.push(field),
                None => break,
            }
            match lexer.next_some()? {
                Token::Punctuation(Punctuation("}")) => break,
                Token::Punctuation(Punctuation(",")) => { },
                _ => return Err(UnexpectedTokenError::TokenMismatch.into()),
            }
        }
        Ok(Struct { name, fields })
    }
    
    /// Parse a single field of struct. Returns `None` if closing brace met instead.
    fn parse_field(lexer: &mut TokenStream) -> Result<Option<Field>, ParserError> {
        let name = match lexer.next_some()? {
            Token::Identifier(ident) => Identifier(ident),
            Token::Punctuation(Punctuation("}")) => return Ok(None),
            _ => return Err(UnexpectedTokenError::TokenMismatch.into()),
        };
        lexer.expect_punctuation(&[":"])?;
        let type_ = Identifier::parse(lexer)?;

        Ok(Some(Field { name, type_ }))
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::TokenStream, parser::expressions::Identifier};

    use super::{Struct, Field};

    #[test]
    fn parse_empty_struct() {
        let mut lexer = TokenStream::new("struct name {}");
        let _ = lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: Vec::new(),
        };
        let produced = Struct::parse(&mut lexer).unwrap();
        assert_eq!(expected, produced);
    }

    #[test]
    fn parse_struct_with_comma() {
        let mut lexer = TokenStream::new("struct name { field1: type1, field2: type2, }");
        let _ = lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: vec![
                Field { name: Identifier(String::from("field1")), type_: Identifier(String::from("type1")) },
                Field { name: Identifier(String::from("field2")), type_: Identifier(String::from("type2")) },
            ]
        };
        let produced = Struct::parse(&mut lexer).unwrap();
        assert_eq!(expected, produced);
    }

    #[test]
    fn parse_struct_without_comma() {
        let mut lexer = TokenStream::new("struct name { field1: type1, field2: type2 }");
        let _ = lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: vec![
                Field { name: Identifier(String::from("field1")), type_: Identifier(String::from("type1")) },
                Field { name: Identifier(String::from("field2")), type_: Identifier(String::from("type2")) },
            ]
        };
        let produced = Struct::parse(&mut lexer).unwrap();
        assert_eq!(expected, produced);
    }
}
