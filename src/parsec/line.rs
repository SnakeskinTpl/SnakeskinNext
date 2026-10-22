use crate::seq;

use super::{choice, input_start, lookup, map, not, parser, tag, take, take_match, ErrorLocation, Parser};

mod err;
pub use err::LineStartError;

#[cfg(test)]
mod tests;

/// Matches the start of input or the position immediately after LF, without consuming.
/// CRLF is recognized after LF; a standalone CR is not a line boundary.
pub fn line_start<'a>() -> impl Parser<'a, Output = ()> {
    choice(input_start(), parser(|input| {
        let position = input.byte_position();

        if input.peek_prev() == Some('\n') {
            Ok(((), input))

        } else {
            Err(LineStartError::new(ErrorLocation {
                start_byte: position,
                failure_byte: position,
                found_char: input.peek(),
            }))
        }
    }))
}

/// Consumes one LF or CRLF line ending and returns its original source slice.
/// A standalone CR is not a line ending.
pub fn line_break<'a>() -> impl Parser<'a, Output = &'a str> {
    choice(tag("\r\n"), tag("\n"))
}

/// Consumes spaces and tabs at the start of a nonblank line, returning their slice.
/// Leaves content unconsumed; blank lines and end of input are mismatches.
pub fn line_indent<'a>() -> impl Parser<'a, Output = &'a str> {
    map(
        seq!(
            line_start(),
            take(|ch, _, _| ch == ' ' || ch == '\t', 0..),
            lookup(take_match(not(line_break()), 1..=1))
        ),
        |(((), indent), _)| indent
    )
}
