//! Lexer

pub mod number;
pub mod punctuation;
pub mod keyword;

use std::{str::FromStr, mem::take};

use thiserror::Error;

use crate::{input_stream::{InputStream, Location}, error::ErrorReporter};

use self::{number::Number, punctuation::{Punctuation, NotPunctuation}, keyword::Keyword};

/// A stream that returns tokens of programming language.
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    /// Cached token.
    current: Option<Token>,
    input: InputStream<'a>,
    pub location: Location,
    pub error_reporter: ErrorReporter,
}

impl<'a> Lexer<'a> {
    pub fn new(input: InputStream<'a>) -> Self {
        let location = input.location();
        Self {
            current: None,
            input,
            location,
            error_reporter: ErrorReporter::new(),
        }
    }

    pub fn error_reporter(&self) -> &ErrorReporter {
        &self.error_reporter
    }

    /// Get next token.
    pub fn next(&mut self) -> Result<Token, LexerError> {
        self.location = self.input.location();
        match take(&mut self.current) {
            Some(token) => Ok(token),
            None => self.read_token(),
        }
    }

    /// Discard next token.
    /// 
    /// That function ignores errors, so it should only be used after successful [peek](Lexer::peek) call.
    pub fn discard(&mut self) {
        let _ = self.next();
    }

    /// Get next token without advancing an iterator.
    pub fn peek(&mut self) -> Result<Token, LexerError> {
        if self.current.is_none() {
            self.current = Some(self.read_token()?);
        }
        Ok(self.current.clone().unwrap())
    }

    /// Check if last token was already yielded.
    pub fn is_eof(&mut self) -> bool {
        matches!(self.peek(), Ok(Token::Eof))
    }

    fn read_token(&mut self) -> Result<Token, LexerError> {
        self.clean();
        let start = self.input.location();
        match self.read_token_inner() {
            Ok(token) => Ok(token),
            Err(err) => {
                self.error_reporter.error()
                    .starts_at(start)
                    .ends_at(self.input.location())
                    .message(err.to_string())
                    .report();
                Err(err)
            },
        }
    }
    
    fn read_token_inner(&mut self) -> Result<Token, LexerError> {
        let ch = match self.input.peek() {
            Some(ch) => ch,
            None => return Ok(Token::Eof),
        };

        if ch == '"' {
            return self.read_str();
        }

        if ch.is_ascii_digit() {
            let number = number::Number::parse(&mut self.input)?;
            return Ok(Token::Number(number));
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.read_identifier();
        }

        if ch.is_ascii_punctuation() {
            return self.read_punctuation();
        }

        Err(LexerError::UnexpectedCharacter(ch))
    }

    /// Remove spaces and comments beforehand.
    fn clean(&mut self) {
        loop {
            let skipped = skip_line_comment(&mut self.input) || skip_block_comment(&mut self.input);
            let skipped = skipped || skip_whitespace(&mut self.input);
            
            if !skipped {
                break;
            }
        }

        fn skip_line_comment(stream: &mut InputStream) -> bool {
            if stream.peek() == Some('/') && stream.peek_nth(1) == Some('/') {
                loop {
                    if let Some('\n') | None = stream.next() {
                        return true;
                    }
                }
            }
            false
        }

        fn skip_block_comment(stream: &mut InputStream) -> bool {
            if stream.peek() == Some('/') && stream.peek_nth(1) == Some('*') {
                stream.next();
                loop {
                    if stream.next() == Some('*') && stream.peek() == Some('/') {
                        stream.next();
                        return true;
                    }

                    if stream.is_eof() {
                        return true;
                    }
                }
            }
            false
        }

        fn skip_whitespace(stream: &mut InputStream) -> bool {
            let mut skipped = false;
            loop {
                let ch = stream.peek();
                if ch.is_some() && ch.unwrap().is_whitespace() {
                    skipped = true;
                    stream.next();
                } else {
                    break;
                }
            }
            skipped
        }
    }

    /// Read string literal.
    fn read_str(&mut self) -> Result<Token, LexerError> {
        self.input.next(); // Skip opening quote mark
        let mut buffer = String::new();
        loop {
            match self.input.next().ok_or(LexerError::UnterminatedString)? {
                '\\' => {
                    let escaped = self.input.next().ok_or(LexerError::UnexpectedEOF)?;
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
        while let Some(ch) = self.input.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                buffer.push(self.input.next().unwrap());
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Punctuation(Punctuation),
    Number(Number),
    String(String),
    Keyword(Keyword),
    Identifier(String),
    Eof,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LexerError {
    #[error("unexpected EOF")]
    UnexpectedEOF,
    #[error("string literal wasn't terminated")]
    UnterminatedString,
    #[error("identifier must contain only ascii alphanumeric and underscore characters")]
    InvalidIdentifier,
    #[error("invalid escape sentence")]
    InvalidEscape,
    #[error("invalid number")]
    InvalidNumber,
    #[error("unknown punctuation")]
    UnknownPunctuation(#[from] NotPunctuation),
    #[error("character `{0}` wasn't expected")]
    UnexpectedCharacter(char),
}

#[cfg(test)]
mod test {
    use crate::{lexer::{
        number::{Base, Number},
        punctuation::Punctuation,
        keyword::Keyword, Token,
    }, input_stream::InputStream};

    use super::Lexer;

    #[test]
    fn return_string() {
        let input = InputStream::new("return \"x > 0\";");
        let mut lexer = Lexer::new(input);

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
        let input = InputStream::new("let x = 123;");
        let mut lexer = Lexer::new(input);

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
            Ok(Token::Punctuation(Punctuation::new("="))),
        );
        assert_eq!(
            lexer.next(),
            Ok(Token::Number(Number {
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
        let input = InputStream::new("if x > 0. { return x; } else { return 0.; }");
        let mut lexer = Lexer::new(input);

        let x = Ok(Token::Identifier(String::from("x")));
        let _return = Ok(Token::Keyword(Keyword::Return));
        let semicolon = Ok(Token::Punctuation(Punctuation::new(";")));
        let zero = Ok(Token::Number(Number {
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
            Ok(Token::Punctuation(Punctuation::new(">"))),
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
