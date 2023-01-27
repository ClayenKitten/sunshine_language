//! Iterator of tokens.

pub mod keyword;
pub mod number;
pub mod operator;
pub mod punctuation;
mod util;

use std::{mem::take, str::FromStr};

use thiserror::Error;

use crate::{
    context::Context,
    input_stream::{InputStream, Location},
    source::SourceId,
};

use self::{
    keyword::Keyword,
    number::Number,
    punctuation::{NotPunctuation, Punctuation},
};

/// A stream that returns tokens of programming language.
#[derive(Debug)]
pub struct Lexer {
    /// Cached token.
    current: Option<Token>,
    input: InputStream,
    pub location: Location,
    pub context: Context,
}

impl Lexer {
    pub fn new(input: InputStream, context: Context) -> Self {
        let location = input.location();
        Self {
            current: None,
            input,
            location,
            context,
        }
    }

    #[cfg(test)]
    pub fn new_test(src: &str) -> Self {
        let input = InputStream::new(src, None);
        Self {
            current: None,
            location: input.location(),
            input,
            context: Context::new_test(),
        }
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
                self.context.error_reporter.lock().unwrap().error(
                    &err,
                    self.source(),
                    start,
                    self.input.location(),
                );
                Err(err)
            }
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
            return Ok(Token::Num(number));
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
        Ok(Token::Str(buffer))
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
            Token::Kw(keyword)
        } else {
            Token::Ident(buffer)
        };
        Ok(token)
    }

    pub fn source(&self) -> Option<SourceId> {
        self.input.source()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Punc(Punctuation),
    Num(Number),
    Str(String),
    Kw(Keyword),
    Ident(String),
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
    use crate::lexer::{
        keyword::Keyword,
        number::{Base, Number},
        punctuation::Punctuation,
        Token,
    };

    use super::Lexer;

    #[test]
    fn return_string() {
        let mut lexer = Lexer::new_test("return \"x > 0\";");

        assert_eq!(lexer.next(), Ok(Token::Kw(Keyword::Return)),);
        assert_eq!(lexer.next(), Ok(Token::Str(String::from("x > 0"))),);
        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new(";"))),);
    }

    #[test]
    fn assign_num_to_var() {
        let mut lexer = Lexer::new_test("let x = 123;");

        assert_eq!(lexer.next(), Ok(Token::Kw(Keyword::Let)),);
        assert_eq!(lexer.next(), Ok(Token::Ident(String::from("x"))),);

        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new("="))),);
        assert_eq!(
            lexer.next(),
            Ok(Token::Num(Number {
                base: Base::Decimal,
                integer: String::from("123"),
                fraction: None,
            })),
        );
        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new(";"))),);
    }

    #[test]
    fn if_with_else() {
        let mut lexer = Lexer::new_test("if x > 0. { return x; } else { return 0.; }");

        let x = Ok(Token::Ident(String::from("x")));
        let _return = Ok(Token::Kw(Keyword::Return));
        let semicolon = Ok(Token::Punc(Punctuation::new(";")));
        let zero = Ok(Token::Num(Number {
            base: Base::Decimal,
            integer: String::from("0"),
            fraction: Some(String::new()),
        }));

        assert_eq!(lexer.next(), Ok(Token::Kw(Keyword::If)));
        assert_eq!(lexer.next(), x);
        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new(">"))),);
        assert_eq!(lexer.next(), zero);

        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new("{"))));
        assert_eq!(lexer.next(), _return);
        assert_eq!(lexer.next(), x);
        assert_eq!(lexer.next(), semicolon);
        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new("}"))));

        assert_eq!(lexer.next(), Ok(Token::Kw(Keyword::Else)));
        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new("{"))));
        assert_eq!(lexer.next(), _return);
        assert_eq!(lexer.next(), zero);
        assert_eq!(lexer.next(), semicolon);
        assert_eq!(lexer.next(), Ok(Token::Punc(Punctuation::new("}"))));
    }
}
