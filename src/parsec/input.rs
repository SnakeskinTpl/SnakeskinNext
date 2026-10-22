#[cfg(test)]
mod tests;

/// A character cursor over borrowed source text.
#[derive(Debug, Clone)]
pub struct ParserInput<'a> {
    source: &'a str,
    byte_pos: usize,
    byte_offset: usize,
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
        Self { source, byte_pos: 0, byte_offset: 0 }
    }

    /// Creates a bounded input slice whose byte positions refer to the enclosing source.
    /// Only text within this slice is accessible to parsing and source_slice.
    pub fn with_offset(source: &'a str, byte_offset: usize) -> Self {
        assert!(byte_offset.checked_add(source.len()).is_some(), "Input byte range overflows usize");
        Self { source, byte_pos: 0, byte_offset }
    }

    /// Byte offset of the next character in the enclosing source, or the slice's end.
    pub fn byte_position(&self) -> usize {
        self.byte_offset + self.byte_pos
    }

    /// Borrows source text independently of the cursor's lifetime.
    /// Uses enclosing-source offsets; panics outside this slice or inside a UTF-8 character.
    pub fn source_slice(&self, range: std::ops::Range<usize>) -> &'a str {
        &self.source[range.start - self.byte_offset..range.end - self.byte_offset]
    }

    pub fn is_at_start(&self) -> bool {
        self.byte_position() == 0
    }

    pub fn is_at_end(&self) -> bool {
        self.byte_pos >= self.source.len()
    }

    /// Reads the next character without advancing the cursor.
    pub fn peek(&self) -> Option<char> {
        self.source[self.byte_pos..].chars().next()
    }

    /// Reads the character before the cursor without moving it; None at input start.
    pub fn peek_prev(&self) -> Option<char> {
        self.source[..self.byte_pos].chars().next_back()
    }

    /// Uses an enclosing-source offset; panics outside this slice or inside a UTF-8 character.
    pub fn set_byte_position(&mut self, byte_pos: usize) {
        let byte_pos = byte_pos.checked_sub(self.byte_offset).expect("Position precedes input slice");
        assert!(self.source.is_char_boundary(byte_pos));
        self.byte_pos = byte_pos;
    }
}
