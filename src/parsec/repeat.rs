use std::ops::RangeBounds;

use super::*;

mod err;
pub use err::{RepeatError, RepeatErrorKind};

#[cfg(test)]
mod tests;

/// Repeats a parser within the given count bounds, restoring input on a failed attempt.
/// Successful iterations must advance the input position.
/// Non-recoverable errors are propagated even after the minimum is met.
/// Panics if the range contains no representable repetition count.
pub fn repeat<'a, P: Parser<'a>>(parser: P, range: impl RangeBounds<usize>) -> Repeat<P> {
    let (min, max) = utils::count_bounds(range).expect("Invalid repetition range");

    Repeat { parser, min, max }
}

#[derive(Debug)]
pub struct Repeat<P> {
    parser: P,
    min: usize,
    max: usize,
}

impl<'a, P: Parser<'a>> Parser<'a> for Repeat<P> {
    type Output = Vec<P::Output>;

    fn parse(&self, mut input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let start = input.byte_position();
        let mut count = 0usize;

        let mut result = Vec::new();

        while count < self.max {
            let (output, remaining) = match self.parser.parse(input.clone()) {
                Ok(result) => result,
                Err(cause) if !cause.is_recoverable() => return Err(cause),
                Err(cause) if count < self.min => {
                    let location = ErrorLocation { start, ..*cause.location() };
                    return Err(RepeatError::insufficient_matches(location, self.min, count, cause));
                }
                Err(_) => break,
            };

            // Reject non-progressing parsers to prevent unbounded repetition.
            if remaining.byte_position() <= input.byte_position() {
                return Err(RepeatError::no_progress(ErrorLocation { start, byte_position: input.byte_position(), actual: input.peek() }));
            }

            result.push(output);
            input = remaining;
            count += 1;
        }

        Ok((result, input))
    }
}
