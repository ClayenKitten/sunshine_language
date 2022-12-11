use std::{str::CharIndices, fmt::{Debug, Display}, cmp::Ordering};

use itertools::{PeekNth, peek_nth};

/// Input stream provides compiler with characters of input and tracks their location.
#[derive(Debug, Clone)]
pub struct InputStream<'a> {
    src: &'a str,
    iter: PeekNth<CharIndices<'a>>,
    // Location of next character.
    location: Location,
}

impl<'a> Iterator for InputStream<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
            .map(|(pos, ch)| {
                self.location.pos = pos + ch.len_utf8();
                if ch == '\n' {
                    self.location.line += 1;
                    self.location.column = 0;
                } else {
                    self.location.column += 1;
                }
                ch
            })
    }
}

impl<'a> InputStream<'a> {
    pub fn new(src: &'a str) -> Self {
        InputStream {
            src,
            iter: peek_nth(src.char_indices()),
            location: Location { pos: 0, line: 0, column: 0 },
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.iter.peek()
            .map(|(_, ch)| *ch)
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<char> {
        self.iter.peek_nth(n)
            .map(|(_, ch)| *ch)
    }

    pub fn is_eof(&mut self) -> bool {
        self.iter.peek().is_none()
    }

    /// Create slice of source code.
    pub fn slice(&mut self, from: Location, to: Location) -> &'a str {
        self.src.get(from.pos .. to.pos)
            .expect("slice is expected to be in boundaries")
    }

    /// Get location of next character.
    pub fn location(&self) -> Location {
        self.location
    }
}

/// Location of character at source code.
#[derive(Debug, Clone, Copy,PartialEq, Eq)]
pub struct Location {
    pos: usize,
    pub line: usize,
    pub column: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.line.cmp(&other.line) {
            Ordering::Equal => {
                self.column.cmp(&other.column).reverse()
            },
            ord => ord.reverse(),
        }
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use crate::input_stream::InputStream;

    #[test]
    fn location() {
        let mut stream = InputStream::new("x = 5;\ny = 2;");
        assert_eq!(0, stream.location.line);
        
        assert_eq!(Some('x'), stream.next());
        assert_eq!(1, stream.location.column);
        
        assert_eq!(Some(';'), stream.nth(4));
        assert_eq!(6, stream.location.column);
        
        assert_eq!(Some('\n'), stream.next());
        assert_eq!(1, stream.location.line);
        assert_eq!(0, stream.location.column);
        
        assert_eq!(Some('y'), stream.next());
        assert_eq!(1, stream.location.line);
        assert_eq!(1, stream.location.column);
    }

    #[test]
    fn slice_one() {
        let mut stream = InputStream::new("123");
        assert_eq!(Some('1'), stream.next());
        let from = stream.location();
        assert_eq!(Some('2'), stream.next());
        let to = stream.location();
        assert_eq!("2", stream.slice(from, to));
    }

    #[test]
    fn slice() {
        let mut stream = InputStream::new("print(\"Hello world\");");
        assert_eq!(Some('('), stream.nth(5));
        let from = stream.location();
        assert_eq!(Some('"'), stream.nth(12));
        let to = stream.location();
        assert_eq!("\"Hello world\"", stream.slice(from, to));
    }
    
    #[test]
    fn slice_unicode() {
        let mut stream = InputStream::new("Привет!:) 😀😀✨! 祝你好运!");
        let location1 = stream.location();
        assert_eq!(Some('!'), stream.nth(6));
        let location2 = stream.location();
        assert_eq!("Привет!", stream.slice(location1, location2));

        assert_eq!(Some(' '), stream.nth(2));
        let location1 = stream.location();
        assert_eq!(Some('!'), stream.nth(3));
        let location2 = stream.location();
        assert_eq!("😀😀✨!", stream.slice(location1, location2));

        assert_eq!(Some(' '), stream.next());
        let location1 = stream.location();
        assert_eq!(Some('!'), stream.nth(4));
        let location2 = stream.location();
        assert_eq!("祝你好运!", stream.slice(location1, location2));
    }
}
