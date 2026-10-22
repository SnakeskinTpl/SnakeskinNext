use super::{ErrorLocation, Parser, ParserInput, ParserResult};

mod err;
pub use err::InputStartError;

#[cfg(test)]
mod tests;

/// Matches byte position zero without consuming input.
pub fn input_start() -> InputStart {
    InputStart
}

#[derive(Debug)]
pub struct InputStart;

impl<'a> Parser<'a> for InputStart {
    type Output = ();

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let position = input.byte_position();

        if input.is_at_start() {
            Ok(((), input))

        } else {
            Err(InputStartError::new(ErrorLocation {
                start_byte: position,
                failure_byte: position,
                found_char: input.peek(),
            }))
        }
    }
}
