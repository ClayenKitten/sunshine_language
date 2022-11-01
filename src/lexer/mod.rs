//! Lexer

pub mod number;
pub mod punctuation;
pub mod keyword;

use std::str::FromStr;

use strum::EnumDiscriminants;
use thiserror::Error;

use crate::input_stream::InputStream;

use self::{number::Number, punctuation::{Operator, Punctuation, NotPunctuation}, keyword::Keyword};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream {
    stream: InputStream,
}

impl TokenStream {
    pub fn new(data: &str) -> Self {
        Self {
            stream: InputStream::new(data),
        }
    }

    /// Get next token if available & succesfully parsed.
    pub fn next(&mut self) -> Result<Token, LexerError> {
        let ch = match self.clean() {
            Some(ch) => ch,
            None => return Ok(Token::Eof),
        };
        self.stream.discard(1);

        if ch == '"' {
            return self.read_str();
        }

        if ch.is_ascii_digit() {
            let number = number::Number::parse(&mut self.stream)?;
            return Ok(Token::Number(number));
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.read_identifier();
        }

        if ch.is_ascii_punctuation() {
            return punctuation::parse(self);
        }

        Err(LexerError::UnexpectedCharacter(ch))
    }

    pub fn is_eof(&self) -> bool {
        self.stream.is_eof()
    }

    /// Remove spaces and comments beforehand.
    fn clean(&mut self) -> Option<char> {
        let stream = &mut self.stream;
        loop {
            // Skip whitespaces
            let mut ch = stream.skip_while(|stream| {
                let ch = stream.peek(0);
                ch.is_some() && ch.unwrap().is_whitespace()
            })?;

            // Skip one line comment
            if ch == '/' && stream.peek(1) == Some('/') {
                stream.skip_while(|stream| stream.peek(0) != Some('\n'));
                ch = stream.next()?;
            }

            // Skip block comment
            if ch == '/' && stream.peek(1) == Some('*') {
                stream.skip_while(|stream| {
                    stream.peek(0) == Some('*') && stream.peek(1) == Some('/')
                });
                stream.next();
                ch = stream.next()?;
            }

            if !ch.is_whitespace()
                && !(ch == '/' && stream.peek(1) == Some('/'))
                && !(ch == '/' && stream.peek(1) == Some('*'))
            {
                return Some(ch);
            }
        }
    }

    /// Read string literal.
    fn read_str(&mut self) -> Result<Token, LexerError> {
        self.stream.next(); // Skip opening quote mark
        let mut buffer = String::new();
        loop {
            match self.stream.next().ok_or(LexerError::UnexpectedEOF)? {
                '\\' => {
                    let escaped = self.stream.next().ok_or(LexerError::UnexpectedEOF)?;
                    let value = match escaped {
                        '\'' => '\'',
                        '"' => '"',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '0' => '\0',
                        _ => return Err(LexerError::InvalidEscape),
                    };
                    buffer.push(value);
                }
                '"' => {
                    break;
                }
                ch => {
                    buffer.push(ch);
                }
            }
        }
        Ok(Token::String(buffer))
    }

    /// Read identifier or keyword.
    fn read_identifier(&mut self) -> Result<Token, LexerError> {
        let mut buffer = String::new();
        while let Some(ch) = self.stream.peek(1) {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                buffer.push(self.stream.next().unwrap());
            } else if !ch.is_ascii() {
                return Err(LexerError::InvalidIdentifier);
            } else {
                break;
            }
        }
        let token = if let Ok(keyword) = Keyword::from_str(&buffer) {
            Token::Keyword(keyword)
        } else {
            Token::Identifier(buffer)
        };
        Ok(token)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(TokenKind))]
pub enum Token {
    Punctuation(Punctuation),
    Operator(Operator),
    Number(Number),
    String(String),
    Keyword(Keyword),
    Identifier(String),
    Eof,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LexerError {
    #[error("Unexpected EOF.")]
    UnexpectedEOF,
    #[error("Identifier must contain only ascii alphanumeric and underscore characters.")]
    InvalidIdentifier,
    #[error("Invalid escape sentence.")]
    InvalidEscape,
    #[error("Invalid number.")]
    InvalidNumber,
    #[error("unknown punctuation")]
    UnknownPunctuation(#[from] NotPunctuation),
    #[error("Character `{0}` wasn't expected.")]
    UnexpectedCharacter(char),
}

#[cfg(test)]
mod test {
    use crate::lexer::{
        number::{Base, Number, Sign},
        punctuation::{Operator, Punctuation},
        keyword::Keyword, Token,
    };

    use super::TokenStream;

    #[test]
    fn return_string() {
        let mut lexer = TokenStream::new("return \"x > 0\";");

        assert_eq!(
            lexer.next(),
            Ok(Token::Keyword(Keyword::Return)),
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::String(String::from("x > 0"))),
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::Punctuation(Punctuation::new(";"))),
        );
    }

    #[test]
    fn assign_num_to_var() {
        let mut lexer = TokenStream::new("let x = 123;");

        assert_eq!(
            lexer.next(),
            Ok(Token::Keyword(Keyword::Let)),
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::Identifier(String::from("x"))),
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::Operator(Operator::Assign)),
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::Number(Number {
                    sign: Sign::Positive,
                    base: Base::Decimal,
                    integer: String::from("123"),
                    fraction: None,
                })
            ),
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::Punctuation(Punctuation::new(";"))),
        );
    }

    #[test]
    fn if_with_else() {
        let mut lexer = TokenStream::new("if x > 0. { return x; } else { return 0.; }");

        let x = Ok(Token::Identifier(String::from("x")));
        let _return = Ok(Token::Keyword(Keyword::Return));
        let semicolon = Ok(Token::Punctuation(Punctuation::new(";")));
        let zero = Ok(Token::Number(Number {
                sign: Sign::Positive,
                base: Base::Decimal,
                integer: String::from("0"),
                fraction: Some(String::new()),
            }),
        );

        assert_eq!(
            lexer.next(),
            Ok(Token::Keyword(Keyword::If))
        );
        assert_eq!(lexer.next(), x);
        assert_eq!(
            lexer.next(),
            Ok(Token::Operator(Operator::More))
        );
        assert_eq!(lexer.next(), zero);

        assert_eq!(
            lexer.next(),
            Ok(Token::Punctuation(Punctuation::new("{")))
        );
        assert_eq!(lexer.next(), _return);
        assert_eq!(lexer.next(), x);
        assert_eq!(lexer.next(), semicolon);
        assert_eq!(
            lexer.next(),
            Ok(Token::Punctuation(Punctuation::new("}")))
        );

        assert_eq!(
            lexer.next(),
            Ok(Token::Keyword(Keyword::Else))
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::Punctuation(Punctuation::new("{")))
        );
        assert_eq!(lexer.next(), _return);
        assert_eq!(lexer.next(), zero);
        assert_eq!(lexer.next(), semicolon);
        assert_eq!(
            lexer.next(),
            Ok(Token::Punctuation(Punctuation::new("}")))
        );
    }
}
