use std::ops::RangeBounds;

use super::{Parser, ParserInput, ParserResult, ErrorLocation, utils};

mod err;
pub use err::TakeMatchError;

#[cfg(test)]
mod tests;

/// Consumes one character per successful parser check and returns a source slice.
/// Checks discard their output and input changes; recoverable failure stops scanning.
/// Non-recoverable errors propagate. Counts are Unicode scalar values, not bytes.
/// EOF or the maximum ends scanning without another check.
/// Panics if the count range is invalid.
pub fn take_match<'a, P: Parser<'a>>(parser: P, range: impl RangeBounds<usize>) -> TakeMatch<P> {
    let (min, max) = utils::count_bounds(range).expect("Invalid character count range");

    TakeMatch { parser, min, max }
}

#[derive(Debug)]
pub struct TakeMatch<P> {
    parser: P,
    min: usize,
    max: usize,
}

impl<'a, P: Parser<'a>> Parser<'a> for TakeMatch<P> {
    type Output = &'a str;

    fn parse(&self, mut input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let start_byte = input.byte_position();
        let mut count = 0usize;

        while count < self.max && input.peek().is_some() {
            match self.parser.parse(input.clone()) {
                Ok(_) => {
                    input.next();
                    count += 1;
                }

                Err(cause) if !cause.is_recoverable() => return Err(cause),

                Err(cause) if count < self.min => {
                    let location = ErrorLocation { start_byte, failure_byte: input.byte_position(), found_char: input.peek() };
                    return Err(TakeMatchError::new(location, self.min, count, Some(cause)));
                }

                Err(_) => break,
            }
        }

        if count < self.min {
            let location = ErrorLocation {
                start_byte,
                failure_byte: input.byte_position(),
                found_char: input.peek(),
            };

            return Err(TakeMatchError::new(location, self.min, count, None));
        }

        Ok((input.source_slice(start_byte..input.byte_position()), input))
    }
}
