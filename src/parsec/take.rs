use std::ops::RangeBounds;

use super::{Parser, ParserInput, ParserResult, ErrorLocation, utils};

mod err;
pub use err::TakeError;

#[cfg(test)]
mod tests;

/// Consumes matching characters and returns a slice of the source.
/// Bounds count Unicode scalar values, not bytes.
/// The predicate receives whether the character is escaped by preceding backslashes.
/// Escape tracking starts fresh for each parse call; backslashes are preserved.
/// Panics if the range contains no representable character count.
pub fn take<P: Fn(char, bool) -> bool>(predicate: P, range: impl RangeBounds<usize>) -> Take<P> {
    let (min, max) = utils::count_bounds(range).expect("Invalid character count range");

    Take { predicate, min, max }
}

#[derive(Debug)]
pub struct Take<P> {
    predicate: P,
    min: usize,
    max: usize,
}

impl<'a, P: Fn(char, bool) -> bool> Parser<'a> for Take<P> {
    type Output = &'a str;

    fn parse(&self, mut input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let start = input.byte_position();
        let mut count = 0usize;
        let mut escaped = false;

        while
            count < self.max &&
            let Some(ch) = input.peek()
        {
            if (self.predicate)(ch, escaped) {
                input.next();
                count += 1;

            } else {
                break;
            }

            if escaped {
                escaped = false;

            } else if ch == '\\' {
                escaped = true;
            }
        }

        if count < self.min {
            let location = ErrorLocation {
                start_byte: start,
                failure_byte: input.byte_position(),
                found_char: input.peek(),
            };

            return Err(TakeError::new(location, self.min, count))
        }

        Ok((input.source_slice(start..input.byte_position()), input))
    }
}
