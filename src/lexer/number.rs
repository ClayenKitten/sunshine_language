use crate::input_stream::InputStream;

use super::LexerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number {
    pub sign: Sign,
    pub integer: String,
    pub fraction: Option<String>,
    pub base: Base,
}

impl Number {
    pub fn parse(stream: &mut InputStream) -> Result<Number, LexerError> {
        let base = Self::parse_base(stream)?;
        let (integer, fraction) = Self::parse_number(stream, base);

        if let Some(fraction) = &fraction {
            if integer.is_empty() && fraction.is_empty() {
                return Err(LexerError::InvalidNumber);
            }
        } else if integer.is_empty() {
            return Err(LexerError::InvalidNumber);
        }

        Ok(Number {
            sign: Sign::Positive,
            base,
            integer,
            fraction,
        })
    }

    fn parse_base(stream: &mut InputStream) -> Result<Base, LexerError> {
        Ok(match stream.next() {
            Some('0') => match stream.next() {
                Some('b') => Base::Binary,
                Some('o') => Base::Octal,
                Some('x') => Base::Hexadecimal,
                Some(_) => {
                    stream.discard(2);
                    Base::Decimal
                }
                None => {
                    stream.discard(1);
                    Base::Decimal
                }
            },
            Some('.' | '1'..='9') => {
                stream.discard(1);
                Base::Decimal
            }
            Some(_) => return Err(LexerError::InvalidNumber),
            None => return Err(LexerError::UnexpectedEOF),
        })
    }

    fn parse_number(stream: &mut InputStream, base: Base) -> (String, Option<String>) {
        let base = match base {
            Base::Binary => 2,
            Base::Octal => 8,
            Base::Decimal => 10,
            Base::Hexadecimal => 16,
        };

        let mut integer = String::new();
        let mut fraction = String::new();
        let mut met_dot = false;

        while let Some(ch) = stream.next() {
            if ch.is_digit(base) {
                if !met_dot {
                    integer.push(ch);
                } else {
                    fraction.push(ch);
                }
            } else if ch == '.' && !met_dot {
                met_dot = true;
            } else {
                stream.discard(1);
                break;
            }
        }

        if met_dot {
            (integer, Some(fraction))
        } else {
            (integer, None)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

#[cfg(test)]
mod test {
    use crate::{
        input_stream::InputStream,
        lexer::number::{Base, Sign},
    };

    use super::Number;

    #[test]
    fn parse_integer() {
        let mut stream = InputStream::new("0");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Decimal,
                integer: String::from("0"),
                fraction: None,
            })
        );

        let mut stream = InputStream::new("1234");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Decimal,
                integer: String::from("1234"),
                fraction: None,
            })
        );

        let mut stream = InputStream::new("0xF422");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Hexadecimal,
                integer: String::from("F422"),
                fraction: None,
            })
        );
    }

    #[test]
    fn parse_float() {
        let mut stream = InputStream::new("1234.56789");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Decimal,
                integer: String::from("1234"),
                fraction: Some(String::from("56789")),
            })
        );

        let mut stream = InputStream::new("0xABC.DEF");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Hexadecimal,
                integer: String::from("ABC"),
                fraction: Some(String::from("DEF")),
            })
        );
    }

    #[test]
    #[should_panic]
    fn invalid_base_binary() {
        let num = Number::parse(&mut InputStream::new("0b2130"));
        num.unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_base_octal() {
        let num = Number::parse(&mut InputStream::new("0o91"));
        num.unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_base_decimal() {
        let num = Number::parse(&mut InputStream::new("ABC"));
        num.unwrap();
    }

    #[test]
    fn half_empty_float_parse() {
        let mut stream = InputStream::new("1234.");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Decimal,
                integer: String::from("1234"),
                fraction: Some(String::new()),
            })
        );

        let mut stream = InputStream::new(".1234");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Decimal,
                integer: String::new(),
                fraction: Some(String::from("1234")),
            })
        );

        let mut stream = InputStream::new("0xABCD.");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Hexadecimal,
                integer: String::from("ABCD"),
                fraction: Some(String::new()),
            })
        );

        let mut stream = InputStream::new("0x.001B");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
                sign: Sign::Positive,
                base: Base::Hexadecimal,
                integer: String::new(),
                fraction: Some(String::from("001B")),
            })
        );
    }
}
