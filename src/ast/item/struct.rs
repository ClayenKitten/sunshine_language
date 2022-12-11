use crate::{ast::{expressions::Identifier}, lexer::Lexer, parser::ParserError};

/// A type that is composed of other types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<Field>,
}

/// Field 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Identifier,
    pub type_: Identifier,
}

impl Struct {
    /// Parse structure from token stream. `struct` keyword is expected to be consumed beforehand.
    #[deprecated = "use Parser::parse_struct"]
    pub fn parse(lexer: &mut Lexer) -> Result<Struct, ParserError> {
        let name = lexer.expect_identifier()?;
        let mut fields = Vec::new();
        lexer.expect_punctuation("{")?;
        
        while let Some(field) = Self::parse_field(lexer)? {
            fields.push(field);
            if lexer.consume_punctuation("}")? {
                break;
            } else {
                lexer.expect_punctuation(",")?;
            }
        }
        Ok(Struct { name, fields })
    }
    
    /// Parse a single field of struct. Returns `None` if closing brace met instead.
    fn parse_field(lexer: &mut Lexer) -> Result<Option<Field>, ParserError> {
        let Some(name) = lexer.consume_identifier()? else {
            lexer.expect_punctuation("}")?;
            return Ok(None);
        };
        lexer.expect_punctuation(":")?;
        let type_ = lexer.expect_identifier()?;

        Ok(Some(Field { name, type_ }))
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, ast::expressions::Identifier, input_stream::InputStream};

    use super::{Struct, Field};

    #[test]
    fn parse_empty_struct() {
        let input = InputStream::new("struct name {}");
        let mut lexer = Lexer::new(input);

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
        let input = InputStream::new("struct name { field1: type1, field2: type2, }");
        let mut lexer = Lexer::new(input);

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
        let input = InputStream::new("struct name { field1: type1, field2: type2 }");
        let mut lexer = Lexer::new(input);

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
