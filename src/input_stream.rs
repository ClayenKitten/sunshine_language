use std::{str::CharIndices, fmt::{Debug, Display}, cmp::Ordering};

use itertools::{PeekNth, peek_nth};

/// Input stream provides compiler with characters of input and tracks their location.
#[derive(Debug, Clone)]
pub struct InputStream<'a> {
    src: &'a str,
    pos: Option<usize>,
    iter: PeekNth<CharIndices<'a>>,
    // Location of next character.
    location: Location,
}

impl<'a> Iterator for InputStream<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
            .map(|(pos, ch)| {
                self.pos = Some(pos);
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
            pos: None,
            iter: peek_nth(src.char_indices()),
            location: Location { line: 0, column: 0 },
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

    /// Store current index in marker.
    /// 
    /// # Returns
    /// 
    /// Returns `None` if iterator wasn't ever advanced yet.
    pub fn mark(&mut self) -> Option<SliceStartMarker> {
        self.pos
            .map(SliceStartMarker)
    }

    /// Create slice of source code.
    /// 
    /// Slice includes both char that was yielded before mark creation
    /// and char that will be yielded on iterator advancement.
    pub fn slice(&mut self, marker: SliceStartMarker) -> &'a str {
        let end_pos = self.iter.peek().copied();
        match end_pos {
            Some((end_pos, _)) => {
                self.src.get(marker.0 ..= end_pos)
                    .expect("slice should be in boundaries")
            },
            None => {
                self.src.get(marker.0..)
                    .expect("slice should be in boundaries")
            },
        }
    }

    /// Get location of next character.
    pub fn location(&self) -> Location {
        self.location
    }
}

/// An index of source that indicates beggining of the slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SliceStartMarker(usize);

/// Location of character at source code.
#[derive(Debug, Clone, Copy,PartialEq, Eq, Ord)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.line.cmp(&other.line) {
            Ordering::Equal => {
                self.column.cmp(&other.column).reverse()
            },
            ord => ord.reverse(),
        })
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
    fn slice() {
        let mut stream = InputStream::new("print(\"Hello world\")");
        assert_eq!(Some('"'), stream.nth(6));
        let marker = stream.mark().unwrap();
        while stream.peek() != Some('"') {
            stream.next();
        }
        assert_eq!("\"Hello world\"", stream.slice(marker));
    }
}
