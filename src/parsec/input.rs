mod state;
pub use state::ParserState;

#[cfg(test)]
mod tests;

#[derive(Debug)]
/// A character cursor and state stack over borrowed source text.
pub struct ParserInput<'a> {
    source: &'a str,
    byte_pos: usize,
    state_stack: Vec<Box<dyn ParserState>>,
}

impl<'a> Clone for ParserInput<'a> {
    // Share immutable source text, but clone the cursor and parser states.
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            byte_pos: self.byte_pos,
            state_stack: self.state_stack.iter().map(|state| state.clone_box()).collect(),
        }
    }
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
        Self { source, byte_pos: 0, state_stack: vec![] }
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

    pub fn is_at_end(&self) -> bool {
        self.byte_pos >= self.source.len()
    }

    /// Reads the next character without advancing the cursor.
    pub fn peek(&self) -> Option<char> {
        self.source[self.byte_pos..].chars().next()
    }

    /// Moves only the cursor; the state stack is unchanged.
    /// Panics if the offset is out of bounds or inside a UTF-8 character.
    pub fn set_byte_position(&mut self, byte_pos: usize) {
        assert!(self.source.is_char_boundary(byte_pos));
        self.byte_pos = byte_pos;
    }

    /// Matches only the top state, requiring both its type and value to match.
    pub fn is_current_state<T: ParserState + PartialEq>(&self, expected: &T) -> bool {
        self.state_stack
            .last()
            .and_then(|state| state.as_any().downcast_ref::<T>())
            .map(|state| state == expected)
            .unwrap_or(false)
    }

    pub fn peek_state(&self) -> Option<&dyn ParserState> {
        self.state_stack.last().map(|state| state.as_ref())
    }

    pub fn push_state(&mut self, state: impl ParserState) {
        self.state_stack.push(Box::new(state));
    }

    pub fn pop_state(&mut self) -> Option<Box<dyn ParserState>> {
        self.state_stack.pop()
    }

    /// Removes and returns the entire stack, preserving its order.
    pub fn take_states(&mut self) -> Vec<Box<dyn ParserState>> {
        std::mem::replace(&mut self.state_stack, vec![])
    }

    /// Installs a stack and returns the previous stack.
    pub fn replace_states(&mut self, state_stack: Vec<Box<dyn ParserState>>) -> Vec<Box<dyn ParserState>> {
        std::mem::replace(&mut self.state_stack, state_stack)
    }
}
