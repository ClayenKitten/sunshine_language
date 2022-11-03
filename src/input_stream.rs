/// Input stream is used to preserve
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputStream {
    pos: usize,
    data: Vec<char>,
}

impl InputStream {
    pub fn new(data: &str) -> Self {
        InputStream {
            pos: 0,
            data: data.chars().collect(),
        }
    }

    /// Get next character.
    pub fn next(&mut self) -> Option<char> {
        let val = self.data.get(self.pos).copied();
        self.pos = self.pos.checked_add(1)?;
        if val.is_none() {
            self.pos -= 1;
        }
        val
    }

    /// Move n character back.
    pub fn discard(&mut self, n: usize) {
        self.pos -= n;
    }

    /// Peek character relative to current position.
    ///
    /// `peek(0)` returns result of last `next()`; `peek(1)` returns result equal to `next()` call.
    pub fn peek(&self, offset: isize) -> Option<char> {
        let index = if offset.is_negative() {
            self.pos.checked_sub(offset.unsigned_abs())
        } else {
            self.pos.checked_add(offset as usize)
        }?
        .checked_sub(1)?;
        self.data.get(index).copied()
    }

    pub fn is_eof(&self) -> bool {
        self.data.len() == self.pos
    }
}

#[cfg(test)]
mod test {
    use super::InputStream;

    #[test]
    fn test_next_equals_peek0() {
        let mut stream = InputStream::new("Hi!");
        let ch1 = stream.next();
        let ch2 = stream.peek(0);
        assert_eq!(ch1, ch2);
    }
}
