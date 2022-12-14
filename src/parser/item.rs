use crate::{
    ast::{
        item::{Field, Function, Item, Module, Parameter, Struct},
        Identifier, Visibility,
    },
    lexer::{keyword::Keyword, punctuation::Punctuation, Token},
};

use super::{FileParser, ParserError, UnexpectedTokenError};

/// [Item]'s parsing.
///
/// [Item]: crate::ast::item::Item
impl FileParser {
    /// Try to parse an item.
    ///
    /// Stores resulting item in parser's [SymbolTable].
    ///
    /// [SymbolTable]: crate::symbol_table::SymbolTable
    pub fn parse_item(&mut self) -> Result<(), ParserError> {
        let start = self.lexer.location;

        let visibility = if self.lexer.consume_keyword(Keyword::Pub)? {
            Visibility::Public
        } else {
            Visibility::default()
        };

        let item = if self.lexer.consume_keyword(Keyword::Fn)? {
            Item::new(self.parse_fn()?, visibility)
        } else if self.lexer.consume_keyword(Keyword::Struct)? {
            Item::new(self.parse_struct()?, visibility)
        } else if self.lexer.consume_keyword(Keyword::Mod)? {
            Item::new(self.parse_module()?, visibility)
        } else {
            let token = self.lexer.next()?;
            self.context.error_reporter
                .lock()
                .unwrap()
                .error()
                .message(String::from("expected an item"))
                .starts_at(start)
                .ends_at(self.lexer.location)
                .report();
            return Err(UnexpectedTokenError::UnexpectedToken(token).into());
        };
        self.symbol_table.declare(self.scope.clone(), item);
        Ok(())
    }

    fn subscope<R>(&mut self, ident: Identifier, func: impl Fn(&mut FileParser) -> R) -> R {
        self.scope.push(ident);
        let result = func(self);
        self.scope.pop();
        result
    }

    /// Parse module. Keyword [mod](Keyword::Mod) is expected to be consumed beforehand.
    pub fn parse_module(&mut self) -> Result<Module, ParserError> {
        let name = self.lexer.expect_identifier()?;
        
        if self.lexer.consume_punctuation(";")? {
            return Ok(Module::Loadable(name))
        }

        self.lexer.expect_punctuation("{")?;
        while !self.lexer.consume_punctuation("}")? {
            self.subscope(name.clone(), |parser| parser.parse_item())?;
        }
        Ok(Module::Inline(name))
    }

    /// Parse toplevel module.
    pub fn parse_top_module(&mut self, name: Identifier) -> Result<Module, ParserError> {
        while !self.lexer.is_eof() {
            self.parse_item()?;
        }
        Ok(Module::Inline(name))
    }

    /// Parse structure. Keyword [struct](Keyword::Struct) is expected to be consumed beforehand.
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

    /// Parse function from token stream. Keyword [fn](Keyword::Fn) is expected to be consumed beforehand.
    pub fn parse_fn(&mut self) -> Result<Function, ParserError> {
        let name = self.lexer.expect_identifier()?;
        self.lexer.expect_punctuation("(")?;
        let params = self.parse_params()?;
        let return_type = self.parse_return_type()?;
        let body = self.subscope(name.clone(), |parser| parser.parse_block())?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    /// Parse parameters. Opening parenthesis is expected to be consumed beforehand.
    fn parse_params(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut params = Vec::new();
        loop {
            let name = match self.lexer.next()? {
                Token::Identifier(ident) => Identifier(ident),
                Token::Punctuation(Punctuation(")")) => break,
                token => return Err(UnexpectedTokenError::UnexpectedToken(token).into()),
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
            }
            Token::Punctuation(Punctuation("{")) => Ok(None),
            token => Err(UnexpectedTokenError::UnexpectedToken(token).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::Identifier, parser::FileParser};

    use super::{Field, Struct};

    #[test]
    fn parse_empty_struct() {
        let mut parser = FileParser::new_test("struct name {}");

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
        let mut parser = FileParser::new_test("struct name { field1: type1, field2: type2, }");

        let _ = parser.lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: vec![
                Field {
                    name: Identifier(String::from("field1")),
                    type_: Identifier(String::from("type1")),
                },
                Field {
                    name: Identifier(String::from("field2")),
                    type_: Identifier(String::from("type2")),
                },
            ],
        };
        let produced = parser.parse_struct().unwrap();
        assert_eq!(expected, produced);
    }

    #[test]
    fn parse_struct_without_comma() {
        let mut parser = FileParser::new_test("struct name { field1: type1, field2: type2 }");

        let _ = parser.lexer.next();
        let expected = Struct {
            name: Identifier(String::from("name")),
            fields: vec![
                Field {
                    name: Identifier(String::from("field1")),
                    type_: Identifier(String::from("type1")),
                },
                Field {
                    name: Identifier(String::from("field2")),
                    type_: Identifier(String::from("type2")),
                },
            ],
        };
        let produced = parser.parse_struct().unwrap();
        assert_eq!(expected, produced);
    }
}
