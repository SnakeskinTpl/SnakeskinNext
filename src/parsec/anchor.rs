use super::{ErrorLocation, Parser, ParserInput, ParserResult};

mod err;
pub use err::{InputEndError, InputStartError};

#[cfg(test)]
mod tests;

/// Matches byte position zero without consuming input.
pub fn input_start() -> InputStart {
    InputStart
}

/// Matches the end of the current input slice without consuming input.
pub fn input_end() -> InputEnd {
    InputEnd
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

#[derive(Debug)]
pub struct InputEnd;

impl<'a> Parser<'a> for InputEnd {
    type Output = ();

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        if input.is_at_end() {
            Ok(((), input))

        } else {
            let position = input.byte_position();

            Err(InputEndError::new(ErrorLocation {
                start_byte: position,
                failure_byte: position,
                found_char: input.peek(),
            }))
        }
    }
}
