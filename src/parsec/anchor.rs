use super::{choice, flat_map, succeed, ErrorLocation, ParseError, Parser, ParserInput, ParserResult};

mod err;
pub use err::{StartOfInputError, LineStartError};

#[cfg(test)]
mod tests;

/// Matches byte position zero without consuming input.
pub fn start_of_input() -> StartOfInput {
    StartOfInput
}

/// Matches the start of input or the position immediately after LF, without consuming.
/// CRLF is recognized after LF; a standalone CR is not a line boundary.
pub fn line_start<'a>() -> impl Parser<'a, Output = ()> {
    choice(start_of_input(), flat_map(succeed(), |_, input| {
        let position = input.byte_position();

        if input.source_slice(0..position).ends_with('\n') {
            Ok(((), input))
        } else {
            Err(LineStartError::new(ErrorLocation {
                start_byte: position,
                failure_byte: position,
                found_char: input.peek(),
            }) as Box<dyn ParseError>)
        }
    }))
}

#[derive(Debug)]
pub struct StartOfInput;

impl<'a> Parser<'a> for StartOfInput {
    type Output = ();

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let position = input.byte_position();

        if input.is_at_start() {
            Ok(((), input))

        } else {
            Err(StartOfInputError::new(ErrorLocation {
                start_byte: position,
                failure_byte: position,
                found_char: input.peek(),
            }))
        }
    }
}
