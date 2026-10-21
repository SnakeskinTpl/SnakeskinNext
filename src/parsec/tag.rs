use super::{Parser, ParserInput, ParserResult, ErrorLocation};

mod err;
pub use err::TagError;

#[cfg(test)]
mod tests;

/// Always succeeds without consuming input.
pub fn succeed() -> Tag<&'static str> {
    tag("")
}

/// Matches an exact prefix and returns a slice of the input source.
/// An empty tag succeeds without advancing the cursor.
pub fn tag<T: AsRef<str>>(tag: T) -> Tag<T> {
    Tag(tag)
}

#[derive(Debug)]
pub struct Tag<T>(T);

impl<'a, T: AsRef<str>> Parser<'a> for Tag<T> {
    type Output = &'a str;

    fn parse(&self, mut input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let tag = self.0.as_ref();
        let start = input.byte_position();

        for expected in tag.chars() {
            let actual = input.peek();

            if actual != Some(expected) {
                let location = ErrorLocation { start_byte: start, failure_byte: input.byte_position(), found_char: actual };
                return Err(TagError::new(location, tag, expected));
            }

            input.next();
        }

        Ok((input.source_slice(start..input.byte_position()), input))
    }
}
