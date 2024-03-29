use crate::{
    ast::item::{Field, Function, Item, ItemKind, Module, Parameter, Struct, Visibility},
    error::{
        library::{lexer::TokenMismatch, parser::ExpectedItem},
        CompilerError, ExpectedToken, ReportProvider,
    },
    lexer::{keyword::Keyword, punctuation::Punctuation, Token},
    util::Span,
    Identifier,
};

use super::{FileParser, PendingFile};

/// [Item]'s parsing.
///
/// [Item]: crate::ast::item::Item
impl FileParser {
    /// Try to parse an item.
    ///
    /// Stores resulting item in parser's [ItemTable].
    ///
    /// [ItemTable]: crate::item_table::ItemTable
    pub fn parse_item(&mut self) -> Result<(), CompilerError> {
        let start = self.location();

        let visibility = if self.lexer.consume_keyword(Keyword::Pub)? {
            Visibility::Public
        } else {
            Visibility::default()
        };

        let item_kind: ItemKind = if self.lexer.consume_keyword(Keyword::Fn)? {
            self.parse_fn()?.into()
        } else if self.lexer.consume_keyword(Keyword::Struct)? {
            self.parse_struct()?.into()
        } else if self.lexer.consume_keyword(Keyword::Mod)? {
            self.parse_module()?.into()
        } else {
            return ExpectedItem::report(self, start).map(|_| unreachable!());
        };

        let span = Span {
            source: self.source(),
            start,
            end: self.location(),
        };

        let item = Item::new(item_kind, span, visibility);

        self.item_table.declare(self.scope.clone(), item);
        Ok(())
    }

    fn subscope<R>(&mut self, ident: Identifier, func: impl Fn(&mut FileParser) -> R) -> R {
        self.scope.push(ident);
        let result = func(self);
        self.scope.pop();
        result
    }

    /// Parse module. Keyword [mod](Keyword::Mod) is expected to be consumed beforehand.
    pub fn parse_module(&mut self) -> Result<Module, CompilerError> {
        let name = self.lexer.expect_identifier()?;

        let start = self.location();
        if self.lexer.consume_punctuation(";")? {
            self.pending.push({
                let mut path = self.scope.clone();
                path.push(name.clone());
                PendingFile::General(path)
            });
            return Ok(Module::Loadable(name));
        }
        if !self.lexer.consume_punctuation("{")? {
            let found = self.lexer.peek()?;
            return TokenMismatch::report(
                self,
                start,
                vec![Punctuation::LBrace.into(), Punctuation::Semicolon.into()],
                found,
            )
            .map(|_| unreachable!());
        }
        while !self.lexer.consume_punctuation("}")? {
            self.subscope(name.clone(), |parser| parser.parse_item())?;
        }
        Ok(Module::Inline(name))
    }

    /// Parse toplevel module.
    pub fn parse_top_module(&mut self, name: Identifier) -> Result<Module, CompilerError> {
        while !self.lexer.is_eof() {
            self.parse_item()?;
        }
        Ok(Module::Inline(name))
    }

    /// Parse structure. Keyword [struct](Keyword::Struct) is expected to be consumed beforehand.
    pub fn parse_struct(&mut self) -> Result<Struct, CompilerError> {
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
    fn parse_field(&mut self) -> Result<Option<Field>, CompilerError> {
        let Some(name) = self.lexer.consume_identifier()? else {
            self.lexer.expect_punctuation("}")?;
            return Ok(None);
        };
        self.lexer.expect_punctuation(":")?;
        let type_ = self.lexer.expect_identifier()?;

        Ok(Some(Field { name, type_ }))
    }

    /// Parse function from token stream. Keyword [fn](Keyword::Fn) is expected to be consumed beforehand.
    pub fn parse_fn(&mut self) -> Result<Function, CompilerError> {
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
    fn parse_params(&mut self) -> Result<Vec<Parameter>, CompilerError> {
        let mut params = Vec::new();
        loop {
            let start = self.location();
            let name = match self.lexer.next()? {
                Token::Ident(ident) => Identifier(ident),
                Token::Punc(Punctuation::RParent) => break,
                token => {
                    return TokenMismatch::report(
                        self,
                        start,
                        vec![ExpectedToken::Identifier, Punctuation::RParent.into()],
                        token,
                    )
                    .map(|_| unreachable!());
                }
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
    fn parse_return_type(&mut self) -> Result<Option<Identifier>, CompilerError> {
        let start = self.location();
        match self.lexer.next()? {
            Token::Punc(Punctuation::Arrow) => {
                let return_type = self.lexer.expect_identifier()?;
                self.lexer.expect_punctuation("{")?;
                Ok(Some(return_type))
            }
            Token::Punc(Punctuation::LBrace) => Ok(None),
            token => TokenMismatch::report(
                self,
                start,
                vec![Punctuation::Arrow.into(), Punctuation::LBrace.into()],
                token,
            )
            .map(|_| unreachable!()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::FileParser, Identifier};

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
