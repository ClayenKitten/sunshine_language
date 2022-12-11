use crate::{ast::{item::{Item, Function, Struct, Module, Field, Parameter}, expressions::Identifier}, lexer::{keyword::Keyword, Token, punctuation::Punctuation}};

use super::{Parser, ParserError, UnexpectedTokenError};

impl<'s> Parser<'s> {
    /// Try to parse an item.
    /// 
    /// Stores resulting item in SymbolTable.
    pub fn parse_item(&mut self) -> Result<(), ParserError> {
        let start = self.lexer.location;
        
        let item = if self.lexer.consume_keyword(Keyword::Fn)? {
            Item::Function(self.parse_fn()?)
        } else if self.lexer.consume_keyword(Keyword::Struct)? {
            Item::Struct(self.parse_struct()?)
        } else if self.lexer.consume_keyword(Keyword::Mod)? {
            Item::Module(self.parse_module()?)
        } else {
            let token = self.lexer.next()?;
            self.error_reporter.error()
                .message(String::from("expected an item"))
                .starts_at(start)
                .ends_at(self.lexer.location)
                .report();
            return Err(UnexpectedTokenError::UnexpectedToken(token).into());
        };

        self.symbol_table.declare(self.scope.clone(), item);
        Ok(())
    }

    /// Parse module. Keyword `mod` is expected to be consumed beforehand.
    pub fn parse_module(&mut self) -> Result<Module, ParserError> {
        let name = self.lexer.expect_identifier()?;
        self.lexer.expect_punctuation("{")?;
        while !self.lexer.consume_punctuation("}")? {
            self.parse_item()?;
        }
        Ok(Module { name })
    }

    /// Parse toplevel module.
    pub fn parse_top_module(&mut self) -> Result<Module, ParserError> {
        while !self.lexer.is_eof() {
            self.parse_item()?;
        }
        Ok(Module { name: Identifier(String::from("TOPLEVEL")) })
    }

    /// Parse structure. keyword `struct` is expected to be consumed beforehand.
    pub fn parse_struct(&mut self) -> Result<Struct, ParserError> {
        let name = self.lexer.expect_identifier()?;
        let mut fields = Vec::new();
        self.lexer.expect_punctuation("{")?;
        
        while let Some(field) = self.parse_field()? {
            fields.push(field);
            if self.lexer.consume_punctuation("}")? {
                break;
            } else {
                self.lexer.expect_punctuation(",")?;
            }
        }
        Ok(Struct { name, fields })
    }
    
    /// Parse a single field of struct. Returns `None` if closing brace met instead.
    fn parse_field(&mut self) -> Result<Option<Field>, ParserError> {
        let Some(name) = self.lexer.consume_identifier()? else {
            self.lexer.expect_punctuation("}")?;
            return Ok(None);
        };
        self.lexer.expect_punctuation(":")?;
        let type_ = self.lexer.expect_identifier()?;

        Ok(Some(Field { name, type_ }))
    }

    /// Parse function from token stream. `fn` keyword is expected to be consumed beforehand.
    pub fn parse_fn(&mut self) -> Result<Function, ParserError> {
        let name = self.lexer.expect_identifier()?;
        self.lexer.expect_punctuation("(")?;
        let params = self.parse_params()?;
        let return_type = self.parse_return_type()?;
        let body = self.parse_block()?;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    /// Parse parameters. Opening parenthesis (`(`) is expected to be consumed beforehand.
    fn parse_params(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut params = Vec::new();
        loop {
            let name = match self.lexer.next()? {
                Token::Identifier(ident) => Identifier(ident),
                Token::Punctuation(Punctuation(")")) => break,
                token => return Err(UnexpectedTokenError::UnexpectedToken(token).into())
            };
            self.lexer.expect_punctuation(":")?;
            let type_ = self.lexer.expect_identifier()?;
            params.push(Parameter { name, type_ });

            if self.lexer.consume_punctuation(")")? {
                break;
            } else {
                self.lexer.expect_punctuation(",")?;
            }
        }
        Ok(params)
    }

    /// Try to parse return type if any. Consumes opening brace `{` which is required for function body.
    fn parse_return_type(&mut self) -> Result<Option<Identifier>, ParserError> {
        match self.lexer.next()? {
            Token::Punctuation(Punctuation("->")) => {
                let return_type = self.lexer.expect_identifier()?;
                self.lexer.expect_punctuation("{")?;
                Ok(Some(return_type))
            },
            Token::Punctuation(Punctuation("{")) => Ok(None),
            token => Err(UnexpectedTokenError::UnexpectedToken(token).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, ast::expressions::Identifier, input_stream::InputStream, parser::Parser};

    use super::{Struct, Field};

    #[test]
    fn parse_empty_struct() {
        let input = InputStream::new("struct name {}");
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let _ = parser.lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: Vec::new(),
        };
        let produced = parser.parse_struct().unwrap();
        assert_eq!(expected, produced);
    }

    #[test]
    fn parse_struct_with_comma() {
        let input = InputStream::new("struct name { field1: type1, field2: type2, }");
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let _ = parser.lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: vec![
                Field { name: Identifier(String::from("field1")), type_: Identifier(String::from("type1")) },
                Field { name: Identifier(String::from("field2")), type_: Identifier(String::from("type2")) },
            ]
        };
        let produced = parser.parse_struct().unwrap();
        assert_eq!(expected, produced);
    }

    #[test]
    fn parse_struct_without_comma() {
        let input = InputStream::new("struct name { field1: type1, field2: type2 }");
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let _ = parser.lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: vec![
                Field { name: Identifier(String::from("field1")), type_: Identifier(String::from("type1")) },
                Field { name: Identifier(String::from("field2")), type_: Identifier(String::from("type2")) },
            ]
        };
        let produced = parser.parse_struct().unwrap();
        assert_eq!(expected, produced);
    }
}
