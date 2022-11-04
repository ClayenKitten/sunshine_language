use std::{str::CharIndices, fmt::{Debug, Display}};

use itertools::{Itertools, PeekNth, peek_nth};

/// Input stream is used to preserve
#[derive(Debug, Clone)]
pub struct InputStream<'a> {
    data: PeekNth<CharIndices<'a>>,
}

impl<'a> Iterator for InputStream<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next()
            .map(|(_, ch)| ch)
    }
}

impl<'a> InputStream<'a> {
    pub fn new(input: &'a str) -> Self {
        InputStream {
            data: peek_nth(input.char_indices()),
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.data.peek()
            .map(|(_, ch)| *ch)
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<char> {
        self.data.peek_nth(n)
            .map(|(_, ch)| *ch)
    }

    pub fn is_eof(&mut self) -> bool {
        self.data.peek().is_none()
    }
}
