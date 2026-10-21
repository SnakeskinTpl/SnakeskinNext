#[cfg(test)]
mod tests;

/// A character cursor over borrowed source text.
#[derive(Debug, Clone)]
pub struct ParserInput<'a> {
    source: &'a str,
    byte_pos: usize,
}

impl Iterator for ParserInput<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.source[self.byte_pos..].chars().next()?;
        self.byte_pos += ch.len_utf8();
        Some(ch)
    }
}

impl<'a> From<&'a str> for ParserInput<'a> {
    fn from(source: &'a str) -> Self {
        Self::new(source)
    }
}

impl<'a> ParserInput<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, byte_pos: 0 }
    }

    /// Byte offset of the next character, or source length at end of input.
    pub fn byte_position(&self) -> usize {
        self.byte_pos
    }

    /// Borrows source text independently of the cursor's lifetime.
    /// Panics for an invalid range or bounds inside a UTF-8 character.
    pub fn source_slice(&self, range: std::ops::Range<usize>) -> &'a str {
        &self.source[range]
    }

    pub fn is_at_start(&self) -> bool {
        self.byte_pos == 0
    }

    pub fn is_at_end(&self) -> bool {
        self.byte_pos >= self.source.len()
    }

    /// Reads the next character without advancing the cursor.
    pub fn peek(&self) -> Option<char> {
        self.source[self.byte_pos..].chars().next()
    }

    /// Panics if the offset is out of bounds or inside a UTF-8 character.
    pub fn set_byte_position(&mut self, byte_pos: usize) {
        assert!(self.source.is_char_boundary(byte_pos));
        self.byte_pos = byte_pos;
    }
}
