use std::ops::RangeBounds;

use super::{utils, ErrorLocation, ParseError, Parser, ParserInput, ParserResult};

mod err;
pub use err::TakeError;

#[cfg(test)]
mod tests;

/// Consumes matching characters and returns a slice of the source.
/// Bounds count Unicode scalar values, not bytes.
/// The predicate receives whether the character is escaped by preceding backslashes.
/// Input is borrowed at the current character, before it is consumed.
/// Escape tracking starts fresh for each parse call; backslashes are preserved.
/// Panics if the range contains no representable character count.
pub fn take<'a, P>(predicate: P, range: impl RangeBounds<usize>) -> impl Parser<'a, Output = &'a str>
where
    P: Fn(char, bool, &ParserInput<'a>) -> bool,
{
    try_take(
        move |ch, escaped, input| Ok(predicate(ch, escaped, input)),
        range,
    )
}

/// Like take, but predicate errors propagate unchanged, even before the minimum.
/// Ok(true) consumes a character; Ok(false) stops before it.
/// The predicate receives escaping state and input positioned at the current character.
/// EOF and the maximum stop scanning without another predicate call.
/// Counts are Unicode scalar values; escape tracking starts fresh on each parse.
/// Panics if the count range is invalid.
pub fn try_take<'a, P>(predicate: P, range: impl RangeBounds<usize>) -> TryTake<P>
where
    P: Fn(char, bool, &ParserInput<'a>) -> Result<bool, Box<dyn ParseError>>,
{
    let (min, max) = utils::count_bounds(range).expect("Invalid character count range");
    TryTake { predicate, min, max }
}

#[derive(Debug)]
pub struct TryTake<P> {
    predicate: P,
    min: usize,
    max: usize,
}

impl<'a, P> Parser<'a> for TryTake<P>
where
    P: Fn(char, bool, &ParserInput<'a>) -> Result<bool, Box<dyn ParseError>>,
{
    type Output = &'a str;

    fn parse(&self, mut input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let start_byte = input.byte_position();
        let mut count = 0usize;
        let mut escaped = false;

        while count < self.max && let Some(ch) = input.peek() {
            if !(self.predicate)(ch, escaped, &input)? {
                break;
            }

            input.next();
            count += 1;
            escaped = !escaped && ch == '\\';
        }

        if count < self.min {
            return Err(TakeError::new(
                ErrorLocation { start_byte, failure_byte: input.byte_position(), found_char: input.peek() },
                self.min,
                count,
            ));
        }

        Ok((input.source_slice(start_byte..input.byte_position()), input))
    }
}
