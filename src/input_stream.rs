use std::{str::CharIndices, fmt::Debug};

use itertools::{PeekNth, peek_nth};

/// Input stream provides compiler with characters of input and tracks their location.
#[derive(Debug, Clone)]
pub struct InputStream<'a> {
    src: &'a str,
    pos: Option<usize>,
    iter: PeekNth<CharIndices<'a>>,
    // Line of next character.
    line: usize,
    // Column of next character.
    column: usize,
}

impl<'a> Iterator for InputStream<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
            .map(|(pos, ch)| {
                self.pos = Some(pos);
                if ch == '\n' {
                    self.line += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
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
            line: 0,
            column: 0,
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
}

/// An index of source that indicates beggining of the slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SliceStartMarker(usize);

#[cfg(test)]
mod test {
    use crate::input_stream::InputStream;

    #[test]
    fn location() {
        let mut stream = InputStream::new("x = 5;\ny = 2;");
        assert_eq!(0, stream.line);
        
        assert_eq!(Some('x'), stream.next());
        assert_eq!(1, stream.column);
        
        assert_eq!(Some(';'), stream.nth(4));
        assert_eq!(6, stream.column);
        
        assert_eq!(Some('\n'), stream.next());
        assert_eq!(1, stream.line);
        assert_eq!(0, stream.column);
        
        assert_eq!(Some('y'), stream.next());
        assert_eq!(1, stream.line);
        assert_eq!(1, stream.column);
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
