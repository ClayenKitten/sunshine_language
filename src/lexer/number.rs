use crate::input_stream::InputStream;

use super::LexerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number {
    pub integer: String,
    pub fraction: Option<String>,
    pub base: Base,
}

impl Number {
    pub fn parse(stream: &mut InputStream) -> Result<Number, LexerError> {
        let base = Self::parse_base(stream);
        let (integer, fraction) = Self::parse_number(stream, base);

        if let Some(fraction) = &fraction {
            if integer.is_empty() && fraction.is_empty() {
                return Err(LexerError::InvalidNumber);
            }
        } else if integer.is_empty() {
            return Err(LexerError::InvalidNumber);
        }

        Ok(Number {
            base,
            integer,
            fraction,
        })
    }

    /// Check for base-defining sequence of characters and return it if found. Returns `Base::Decimal` if sequence wasn't found.
    fn parse_base(stream: &mut InputStream) -> Base {
        if stream.peek() != Some('0') {
            return Base::Decimal;
        }

        let base = match stream.peek_nth(1) {
            Some('b') => Base::Binary,
            Some('o') => Base::Octal,
            Some('x') => Base::Hexadecimal,
            _ => return Base::Decimal,
        };

        stream.next();
        stream.next();

        base
    }

    fn parse_number(stream: &mut InputStream, base: Base) -> (String, Option<String>) {
        let mut integer = String::new();
        let mut fraction = String::new();
        let mut met_dot = false;

        while let Some(ch) = stream.peek() {
            if ch.is_digit(base.radix()) {
                if !met_dot {
                    integer.push(ch);
                } else {
                    fraction.push(ch);
                }
                stream.next();
            } else if ch == '.' && !met_dot {
                met_dot = true;
                stream.next();
            } else {
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
pub enum Base {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl Base {
    /// Get radix of base in numerical form.
    pub fn radix(&self) -> u32 {
        match self {
            Base::Binary => 2,
            Base::Octal => 8,
            Base::Decimal => 10,
            Base::Hexadecimal => 16,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{input_stream::InputStream, lexer::number::Base};

    use super::Number;

    #[test]
    fn parse_integer() {
        let mut stream = InputStream::new("0");
        let sign = Number::parse(&mut stream);
        assert_eq!(
            sign,
            Ok(Number {
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
                base: Base::Hexadecimal,
                integer: String::new(),
                fraction: Some(String::from("001B")),
            })
        );
    }
}
